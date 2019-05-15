mod binary_message;
mod text_message;
mod worker;

use self::{binary_message::*, text_message::*, worker::*};
use actix::prelude::*;
use actix_web_actors::ws;
use crossbeam::channel::{self, Receiver, Sender};
use log::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
  collections::HashMap,
  path::PathBuf,
  sync::{Arc, Mutex},
  thread,
};
use time::Duration;
use timer::*;

pub struct WebSocketActor {
  root_path: Vec<String>,
  current_dir: Vec<String>,
  thread_control_sender: Option<Sender<ThreadControlMessage>>,
}

impl WebSocketActor {
  pub fn new(root_path: &PathBuf) -> Self {
    Self {
      root_path: get_components(root_path),
      current_dir: Vec::new(),
      thread_control_sender: None,
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
}

impl Actor for WebSocketActor {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    info!("ws started");

    let (thread_control_sender, thread_control_receiver) = channel::unbounded();
    self.thread_control_sender = Some(thread_control_sender);

    let (event_sender, event_receiver) = channel::unbounded();

    // send initial current directory entries
    self.change_dir(Vec::new());

    start_event_sender_thread(ctx.address(), event_receiver);

    start_scanner_thread(
      self.root_path.clone(),
      thread_control_receiver,
      event_sender,
    );
  }

  fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
    info!("ws stopping");

    Running::Stop
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {
    info!("ws stopped");
  }
}

impl Handler<TextMessage> for WebSocketActor {
  type Result = ();

  fn handle(&mut self, msg: TextMessage, ctx: &mut Self::Context) -> Self::Result {
    ctx.text(msg.0);
  }
}

impl Handler<BinaryMessage> for WebSocketActor {
  type Result = ();

  fn handle(&mut self, msg: BinaryMessage, ctx: &mut Self::Context) -> Self::Result {
    ctx.binary(msg.0);
  }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for WebSocketActor {
  fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
    match msg {
      ws::Message::Text(text) => {
        let control_message: ControlMessage = serde_json::from_str(&text).unwrap();
        info!("{:#?}", control_message);

        match control_message {
          ControlMessage::ChangeDirectory { path } => {
            self.change_dir(path);
          }
        }
      }

      ws::Message::Close(_) => ctx.stop(),

      _ => {
        warn!("other ws kind! {:#?}", msg);
      }
    }
  }
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum EventMessage {
  DirectoryChange {
    path: Vec<String>,
    entries: Vec<Entry>,
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
}


fn start_event_sender_thread(
  addr: Addr<WebSocketActor>,
  event_receiver: Receiver<EventMessage>,
) -> thread::JoinHandle<()> {
  thread::Builder::new()
    .name("event_sender".to_string())
    .spawn(move || {
      let sums_mutex: Arc<Mutex<HashMap<String, u64>>> = Arc::new(Mutex::new(HashMap::new()));
      let timers_mutex: Arc<Mutex<HashMap<String, Guard>>> = Arc::new(Mutex::new(HashMap::new()));
      // TODO combine these into (sum, guard)

      let timer = Timer::new();

      for event in event_receiver {

        let mut sums = sums_mutex.lock().unwrap();
        let mut timers = timers_mutex.lock().unwrap();

        match &event {
          EventMessage::DirectoryChange { .. } => {
            // clear maps
            sums.clear();
            timers.clear();
          }

          EventMessage::SizeUpdate { entry } => {
            // push to sums, timer somehow
            let (name, size) = match entry {
              Entry::Directory { name, size } => (name.clone(), *size),
              Entry::File { name, size } => (name.clone(), *size),
            };

            if !sums.contains_key(&name) {
              timers.insert(name.clone(), {
                let name = name.clone();
                let timers_mutex = timers_mutex.clone();
                let sums_mutex = sums_mutex.clone();
                let addr = addr.clone();

                timer.schedule_with_delay(Duration::milliseconds(1000), move || {
                  {
                    let mut timers = timers_mutex.lock().unwrap();
                    timers.remove(&name);
                  }

                  let mut sums = sums_mutex.lock().unwrap();
                  let size = sums.remove(&name).unwrap();

                  addr.do_send(TextMessage(
                    serde_json::to_string(&EventMessage::SizeUpdate {
                      entry: Entry::Directory {
                        name: name.clone(),
                        size,
                      },
                    })
                    .unwrap(),
                  ));
                })
              });
            }

            sums
              .entry(name)
              .and_modify(move |old_size| {
                *old_size += size;
              })
              .or_insert_with(move || size);

            continue;
          }
        }


        addr.do_send(TextMessage(serde_json::to_string(&event).unwrap()));
      }

    })
    .unwrap()
}
