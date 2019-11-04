mod worker;

use self::worker::*;
use crate::error::*;
use futures::{
  channel::mpsc::unbounded as unbounded_stream, future::RemoteHandle, lock::Mutex, prelude::*,
};
use log::*;
use serde::{Deserialize, Serialize};
use std::{
  collections::HashMap,
  fs,
  path::PathBuf,
  sync::{
    mpsc::{channel, Sender},
    Arc,
  },
  time::Duration,
};

pub struct WebsocketHandler {
  root_path: Vec<String>,
  current_dir: Vec<String>,
  thread_control_sender: Option<Sender<ThreadControlMessage>>,
}

impl WebsocketHandler {
  pub fn new(root_path: &PathBuf) -> Self {
    Self {
      root_path: get_components(root_path),
      current_dir: Vec::new(),
      thread_control_sender: None,
    }
  }

  fn handle_message(&mut self, text: &str) {
    let control_message: ControlMessage = serde_json::from_str(text).unwrap();

    match control_message {
      ControlMessage::ChangeDirectory { path } => {
        self.change_dir(path);
      }

      ControlMessage::Delete { path } => {
        self.delete(path);
      }
    }
  }

  fn change_dir(&mut self, path: Vec<String>) {
    self.current_dir = path.clone();

    self
      .thread_control_sender
      .as_ref()
      .unwrap()
      .send(ThreadControlMessage::ChangeDirectory(path))
      .unwrap();
  }

  fn delete(&mut self, path: Vec<String>) {
    let full_path: PathBuf = self.root_path.iter().cloned().chain(path).collect();
    info!("delete {:?}", full_path);

    let metadata = fs::metadata(&full_path).unwrap();
    if metadata.is_dir() {
      fs::remove_dir_all(full_path).unwrap();
    } else {
      fs::remove_file(full_path).unwrap();
    }

    self.refresh();
  }

  fn refresh(&mut self) {
    self.change_dir(self.current_dir.clone());
  }

  pub async fn start(&mut self, ws: warp::ws::WebSocket) -> Result<()> {
    info!("ws started");

    let (ws_sink, mut ws_stream) = ws.split();

    let (thread_control_sender, thread_control_receiver) = channel();
    self.thread_control_sender = Some(thread_control_sender);

    let (event_sender, event_receiver) = unbounded_stream();

    // send initial current directory entries
    self.change_dir(Vec::new());

    tokio::spawn(async move {
      let event_receiver = start_event_sender_thread(event_receiver);

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
    );

    while let Some(message) = ws_stream.next().await {
      let message = message.chain_err(|| "ws_stream message error")?;
      if message.is_text() {
        let text = message.to_str().unwrap();
        self.handle_message(text);
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

fn start_event_sender_thread<T>(mut event_receiver: T) -> impl Stream<Item = EventMessage>
where
  T: Stream<Item = EventMessage> + Unpin + Send + 'static,
{
  let (mut sender, receiver) = unbounded_stream();

  tokio::spawn(async move {
    #[allow(clippy::type_complexity)]
    let sums_mutex: Arc<Mutex<HashMap<String, (u64, RemoteHandle<()>)>>> =
      Arc::new(Mutex::new(HashMap::new()));

    let sums_mutex2 = sums_mutex.clone();

    while let Some(event) = event_receiver.next().await {
      let mut sums = sums_mutex.lock().await;

      match &event {
        EventMessage::DirectoryChange { .. } => {
          // clear maps
          sums.clear();
        }

        EventMessage::SizeUpdate { entry } => {
          // push to sums, timer somehow
          let (name, size) = match entry {
            Entry::Directory { name, size } | Entry::File { name, size } => (name.clone(), *size),
          };

          let sums_mutex = sums_mutex2.clone();
          let mut sender = sender.clone();

          sums
            .entry(name.clone())
            .and_modify(|(old_size, _remote_handle)| {
              *old_size = size;
            })
            .or_insert_with(move || {
              let fut = async move {
                tokio::timer::delay_for(Duration::from_millis(UPDATE_INTERVAL)).await;

                let mut sums = sums_mutex.lock().await;
                let (size, _remote_handle) = sums.remove(&name).unwrap();

                sender
                  .send(EventMessage::SizeUpdate {
                    entry: Entry::Directory {
                      name: name.clone(),
                      size,
                    },
                  })
                  .await
                  .unwrap()
              };

              let (fut, remote_handle) = fut.remote_handle();

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
