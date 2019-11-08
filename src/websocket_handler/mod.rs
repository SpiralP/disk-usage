mod worker;

use self::worker::*;
use crate::error::*;
use futures::{
  channel::mpsc::{unbounded as unbounded_stream, UnboundedSender},
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

    let (ws_sink, mut ws_stream) = ws.split();

    let (thread_control_sender, thread_control_receiver) = unbounded_stream();
    self.thread_control_sender = Some(thread_control_sender);

    let (event_sender, event_receiver) = unbounded_stream();

    // send initial current directory entries
    self.change_dir(Vec::new()).await;

    tokio::spawn(async move {
      let event_receiver = spawn_size_update_stream(event_receiver);

      event_receiver
        .map(Ok)
        .forward(ws_sink.with(|event: EventMessage| {
          async move {
            let s = serde_json::to_string(&event).unwrap();
            Ok::<_, warp::Error>(warp::ws::Message::text(s))
          }
        }))
        .await
        .unwrap();
    });

    start_scanner_thread(
      self.root_path.clone(),
      thread_control_receiver,
      event_sender,
    )
    .await;

    while let Some(message) = ws_stream.next().await {
      let message = message.chain_err(|| "ws_stream message error")?;
      if message.is_text() {
        let text = message.to_str().unwrap();
        self.handle_message(text).await;
      } else if message.is_close() {
        println!("CLOSE");
      }
    }

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

fn spawn_size_update_stream<T>(mut event_receiver: T) -> impl Stream<Item = EventMessage>
where
  T: Stream<Item = EventMessage> + Unpin + Send + 'static,
{
  let (mut sender, receiver) = unbounded_stream();

  tokio::spawn(async move {
    #[allow(clippy::type_complexity)]
    let sums_mutex = Arc::new(Mutex::new(HashMap::new()));

    let sums_mutex2 = sums_mutex.clone();

    while let Some(event) = event_receiver.next().await {
      let mut sums = sums_mutex.lock().await;

      match &event {
        EventMessage::DirectoryChange { .. } => {
          // clear maps, stop timers!!
          sums.clear();
        }

        EventMessage::SizeUpdate { entry } => {
          // push to sums, send size update message every interval
          let (name, size) = match entry {
            Entry::Directory { name, size } | Entry::File { name, size } => (name.clone(), *size),
          };

          sums
            .entry(name.clone())
            .and_modify(|(old_size, _remote_handle)| {
              // always modify size
              *old_size = size;
            })
            .or_insert_with(|| {
              // if no entry then start timer
              let sums_mutex = sums_mutex2.clone();
              let mut sender = sender.clone();

              let fut = async move {
                tokio::timer::delay_for(Duration::from_millis(UPDATE_INTERVAL)).await;

                let mut sums = sums_mutex.lock().await;
                let (size, my_remote_handle) = sums.remove(&name).unwrap();

                sender
                  .send(EventMessage::SizeUpdate {
                    entry: Entry::Directory {
                      name: name.clone(),
                      size,
                    },
                  })
                  .await
                  .unwrap();

                drop(my_remote_handle);
              };

              let (fut, remote_handle) = fut.remote_handle();
              // we will drop remote_handle (and stop the timer) on dir change "sums.clear()"
              tokio::spawn(fut);

              (size, remote_handle)
            });

          continue;
        }
      }

      sender.send(event).await.unwrap();
    }
  });

  receiver
}
