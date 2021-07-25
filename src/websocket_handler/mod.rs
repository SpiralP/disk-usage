mod api;
mod worker;

use self::{
    api::{DeletingStatus, Entry, EventMessage, UpdatingStatus},
    worker::{get_components, spawn_scanner_stream, ScannerControlMessage},
};
use crate::websocket_handler::api::ControlMessage;
use failure::{Error, ResultExt};
use futures::{
    channel::mpsc::{unbounded as unbounded_stream, UnboundedReceiver, UnboundedSender},
    future,
    future::Either,
    lock::Mutex,
    prelude::*,
};
use log::{debug, info, warn};
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc, thread, time::Duration};

pub struct WebsocketHandler {
    root_path: Vec<String>,
    current_dir: Vec<String>,
    thread_control_sender: UnboundedSender<ScannerControlMessage>,
    event_sender: UnboundedSender<EventMessage>,
}

impl WebsocketHandler {
    async fn handle_message(&mut self, text: &str) {
        let control_message: ControlMessage = serde_json::from_str(text).unwrap();

        match control_message {
            ControlMessage::ChangeDirectory { path } => {
                self.change_dir(path).await;
            }

            ControlMessage::Delete { path } => {
                if let Err(err) = self.delete(path.clone()).await {
                    warn!("couldn't delete path {:?}: {}", path, err);
                }
            }

            ControlMessage::Reveal { path } => {
                let full_path: PathBuf = self.root_path.iter().cloned().chain(path).collect();
                debug!("Reveal {:?}", full_path);

                thread::spawn(move || {
                    if let Err(err) = reveal::that(&full_path) {
                        // TODO make these show in the browser
                        warn!("couldn't reveal path {:?}: {}", full_path, err);
                    }
                });
            }
        }
    }

    async fn change_dir(&mut self, path: Vec<String>) {
        self.current_dir = path.clone();

        self.thread_control_sender
            .send(ScannerControlMessage::ChangeDirectory(path))
            .await
            .unwrap();
    }

    async fn delete(&mut self, path: Vec<String>) -> Result<(), Error> {
        let full_path: PathBuf = self.root_path.iter().cloned().chain(path.clone()).collect();
        info!("delete {:?}", full_path);

        // TODO should this just go in the js?
        let (delay_future, delay_remote_handle) =
            tokio::time::sleep(std::time::Duration::from_secs(1))
                .boxed()
                .remote_handle();
        let either = future::select(
            delay_future,
            tokio::task::spawn_blocking(|| {
                let metadata = fs::metadata(&full_path)?;
                if metadata.is_dir() {
                    #[cfg(windows)]
                    remove_dir_all::remove_dir_all(full_path)?;

                    #[cfg(not(windows))]
                    fs::remove_dir_all(full_path)?;
                } else {
                    fs::remove_file(full_path)?;
                }

                Ok::<_, std::io::Error>(())
            }),
        )
        .await;

        match either {
            Either::Left((_, remove_future)) => {
                // we're taking a long time, start notifying client

                self.send_event(EventMessage::Deleting {
                    path: path.clone(),
                    status: DeletingStatus::Deleting,
                })
                .await
                .with_context(|_| "self.send_event()")
                .unwrap();

                remove_future
                    .await
                    .with_context(|_| "remove_future panic?")
                    .unwrap()?;
            }

            Either::Right((ret, _ignore_delay_future)) => {
                // delete finished quickly, don't tell client anything

                // stop timer
                drop(delay_remote_handle);
                ret.with_context(|_| "remove_future panic?").unwrap()?;
            }
        }

        // always tell the client we're done deleting
        self.send_event(EventMessage::Deleting {
            path,
            status: DeletingStatus::Finished,
        })
        .await
        .unwrap();

        self.refresh().await;

        Ok(())
    }

    async fn refresh(&mut self) {
        self.change_dir(self.current_dir.clone()).await;
    }

    async fn send_event(&mut self, event: EventMessage) -> Result<(), Error> {
        self.event_sender
            .send(event)
            .await
            .with_context(|_| "event_sender.send()")?;

        Ok(())
    }

    pub async fn run(root_path: &PathBuf, ws: warp::ws::WebSocket) {
        info!("ws started");

        let root_path = get_components(root_path);

        // scanner -> size_update -> ws_sender

        let (thread_control_sender, thread_control_receiver) = unbounded_stream();

        let event_receiver = spawn_scanner_stream(root_path.clone(), thread_control_receiver).await;

        let (event_sender, mut event_receiver) = spawn_size_update_stream(event_receiver);

        let (mut ws_sender, mut ws_receiver) = ws.split();

        let mut handler = WebsocketHandler {
            root_path,
            current_dir: Vec::new(),
            thread_control_sender,
            event_sender,
        };

        let ws_sender_future = async move {
            while let Some(event) = event_receiver.next().await {
                let s = serde_json::to_string(&event).unwrap();
                let message = warp::ws::Message::text(s);
                if let Err(e) = ws_sender.send(message).await {
                    warn!("ws_sender.send(): {}", e);
                    break;
                }
            }

            debug!("ws_sender completed");
        }
        .boxed();

        let ws_receiver_future = async move {
            while let Some(maybe_message) = ws_receiver.next().await {
                match maybe_message {
                    Ok(message) => {
                        if message.is_text() {
                            let text = message.to_str().unwrap();
                            handler.handle_message(text).await;
                        } else if message.is_close() {
                            debug!("ws close");
                            break;
                        }
                    }

                    Err(e) => {
                        warn!("ws_receiver.next(): {}", e);
                        break;
                    }
                }
            }

            debug!("ws_receiver completed");
        }
        .boxed();

        // race the websocket sender and receiver to determine close
        future::select(ws_sender_future, ws_receiver_future).await;
    }
}

const UPDATE_INTERVAL: u64 = 500;

fn spawn_size_update_stream(
    mut event_receiver: UnboundedReceiver<EventMessage>,
) -> (
    UnboundedSender<EventMessage>,
    UnboundedReceiver<EventMessage>,
) {
    let (mut event_sender, new_event_receiver) = unbounded_stream();

    let new_event_sender = event_sender.clone();

    tokio::spawn(async move {
        #[allow(clippy::type_complexity)]
        let sums_mutex = Arc::new(Mutex::new(HashMap::new()));
        let sums_mutex_weak = Arc::downgrade(&sums_mutex);

        while let Some(event) = event_receiver.next().await {
            match &event {
                EventMessage::DirectoryChange { .. } => {
                    // clear maps, stop timers!!
                    sums_mutex.lock().await.clear();
                }

                EventMessage::SizeUpdate { entry } => {
                    // push to sums, send size update message every interval
                    let (path, size, updating) = match entry {
                        Entry::File { .. } => unreachable!(),
                        Entry::Directory {
                            path,
                            size,
                            updating,
                        } => (path.clone(), *size, *updating),
                    };

                    let slow_update = match updating {
                        UpdatingStatus::Updating => size != 0,
                        _ => false,
                    };

                    if slow_update {
                        // If it's not a start/finish message,
                        // we update slowly

                        sums_mutex
                            .lock()
                            .await
                            .entry(path.clone())
                            .and_modify(|(old_size, _remote_handle)| {
                                // always modify size
                                *old_size = size;
                            })
                            .or_insert_with(|| {
                                // if no entry then start timer
                                let sums_mutex_weak = sums_mutex_weak.clone();
                                let mut event_sender = event_sender.clone();

                                let (fut, remote_handle) = async move {
                                    tokio::time::sleep(Duration::from_millis(UPDATE_INTERVAL))
                                        .await;

                                    // this upgrade shouldn't fail because when spawn_size_update_stream's remote handle is dropped,
                                    // the only true reference to sums_mutex will be dropped,
                                    // causing this timer future to be dropped and stop running
                                    let sums_mutex = sums_mutex_weak
                                        .upgrade()
                                        .expect("sums_mutex_weak.upgrade shouldn't happen??");
                                    let mut sums = sums_mutex.lock().await;
                                    let (size, my_remote_handle) = sums.remove(&path).unwrap();

                                    if let Err(e) = event_sender
                                        .send(EventMessage::SizeUpdate {
                                            entry: Entry::Directory {
                                                path: path.clone(),
                                                size,
                                                updating,
                                            },
                                        })
                                        .await
                                    {
                                        warn!("timer size_update: {}", e);
                                    }

                                    drop(my_remote_handle);
                                }
                                .remote_handle();
                                // we will drop remote_handle (and stop the timer) on dir change "sums.clear()"
                                tokio::spawn(fut);

                                (size, remote_handle)
                            });

                        // don't send this message
                        continue;
                    } else {
                        // TODO 0 size folders spam because 0 size and updating: true
                        // most probably aren't 0 tho

                        // remove timer so we don't undo our updating: false
                        sums_mutex.lock().await.remove(&path);
                    }
                }

                _ => {}
            }

            if let Err(e) = event_sender.send(event).await {
                warn!("size_update: {}", e);
            }
        } // while event_receiver

        debug!("size_update event_receiver completed");
    });

    (new_event_sender, new_event_receiver)
}
