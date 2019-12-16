mod worker;

use self::worker::*;
use crate::error::*;
use futures::{
  channel::mpsc::{unbounded as unbounded_stream, UnboundedReceiver, UnboundedSender},
  lock::Mutex,
  prelude::*,
};
use log::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc, time::Duration};

pub struct WebsocketHandler {
  root_path: Vec<String>,
  current_dir: Vec<String>,
  thread_control_sender: Option<UnboundedSender<ScannerControlMessage>>,
}

impl WebsocketHandler {
  pub fn new(root_path: &PathBuf) -> Self {
    Self {
      root_path: get_components(root_path),
      current_dir: Vec::new(),
      thread_control_sender: None,
    }
  }

  async fn handle_message(&mut self, text: &str) {
    let control_message: ControlMessage = serde_json::from_str(text).unwrap();

    match control_message {
      ControlMessage::ChangeDirectory { path } => {
        self.change_dir(path).await;
      }

      ControlMessage::Delete { path } => {
        self.delete(path).await;
      }
    }
  }

  async fn change_dir(&mut self, path: Vec<String>) {
    self.current_dir = path.clone();

    self
      .thread_control_sender
      .as_ref()
      .unwrap()
      .send(ScannerControlMessage::ChangeDirectory(path))
      .await
      .unwrap();
  }

  async fn delete(&mut self, path: Vec<String>) {
    let full_path: PathBuf = self.root_path.iter().cloned().chain(path).collect();
    info!("delete {:?}", full_path);

    let metadata = fs::metadata(&full_path).unwrap();
    if metadata.is_dir() {
      fs::remove_dir_all(full_path).unwrap();
    } else {
      fs::remove_file(full_path).unwrap();
    }

    self.refresh().await;
  }

  async fn refresh(&mut self) {
    self.change_dir(self.current_dir.clone()).await;
  }

  pub async fn start(&mut self, ws: warp::ws::WebSocket) -> Result<()> {
    info!("ws started");

    let (mut ws_sender, mut ws_receiver) = ws.split();

    let (thread_control_sender, thread_control_receiver) = unbounded_stream();
    self.thread_control_sender = Some(thread_control_sender);

    // scanner -> size_update -> ws_sender

    let event_receiver =
      spawn_scanner_stream(self.root_path.clone(), thread_control_receiver).await;

    let mut event_receiver = spawn_size_update_stream(event_receiver);

    let ws_sender_future = async move {
      while let Some(event) = event_receiver.next().await {
        let s = serde_json::to_string(&event).unwrap();
        let message = warp::ws::Message::text(s);
        if let Err(e) = ws_sender.send(message).await {
          warn!("ws_sender: {}", e);
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
              self.handle_message(text).await;
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

    info!("ws stopped");

    Ok(())
  }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum EventMessage {
  DirectoryChange {
    path: Vec<String>,
    entries: Vec<Entry>,
    free: u64,
  },

  SizeUpdate {
    /// always Entry::Directory
    entry: Entry,
  },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
enum ControlMessage {
  ChangeDirectory { path: Vec<String> },
  Delete { path: Vec<String> },
}

const UPDATE_INTERVAL: u64 = 500;

fn spawn_size_update_stream(
  mut event_receiver: UnboundedReceiver<EventMessage>,
) -> UnboundedReceiver<EventMessage> {
  let (mut event_sender, new_event_receiver) = unbounded_stream();

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
          let (name, size, updating) = match entry {
            Entry::File { .. } => unreachable!(),
            Entry::Directory {
              name,
              size,
              updating,
            } => (name.clone(), *size, *updating),
          };

          if updating && size != 0 {
            // If it's not a start/finish message,
            // we update slowly

            sums_mutex
              .lock()
              .await
              .entry(name.clone())
              .and_modify(|(old_size, _remote_handle)| {
                // always modify size
                *old_size = size;
              })
              .or_insert_with(|| {
                // if no entry then start timer
                let sums_mutex_weak = sums_mutex_weak.clone();
                let mut event_sender = event_sender.clone();

                let (fut, remote_handle) = async move {
                  tokio::timer::delay_for(Duration::from_millis(UPDATE_INTERVAL)).await;

                  // this upgrade shouldn't fail because when spawn_size_update_stream's remote handle is dropped,
                  // the only true reference to sums_mutex will be dropped,
                  // causing this timer future to be dropped and stop running
                  let sums_mutex = sums_mutex_weak
                    .upgrade()
                    .expect("sums_mutex_weak.upgrade shouldn't happen??");
                  let mut sums = sums_mutex.lock().await;
                  let (size, my_remote_handle) = sums.remove(&name).unwrap();

                  if let Err(e) = event_sender
                    .send(EventMessage::SizeUpdate {
                      entry: Entry::Directory {
                        name: name.clone(),
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
            sums_mutex.lock().await.remove(&name);
          }
        }
      }

      if let Err(e) = event_sender.send(event).await {
        warn!("size_update: {}", e);
      }
    } // while event_receiver

    debug!("size_update event_receiver completed");
  });

  new_event_receiver
}
