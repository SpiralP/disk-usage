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
use std::{path::PathBuf, thread};

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

fn start_event_sender_thread(
  addr: Addr<WebSocketActor>,
  event_receiver: Receiver<EventMessage>,
) -> thread::JoinHandle<()> {
  thread::spawn(move || {
    for event in event_receiver {
      addr.do_send(TextMessage(serde_json::to_string(&event).unwrap()));
    }
  })
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
