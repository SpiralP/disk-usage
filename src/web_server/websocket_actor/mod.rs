mod binary_message;
mod dir;
mod text_message;
mod tree;

use self::{binary_message::*, dir::*, text_message::*, tree::*};
use actix::prelude::*;
use actix_web_actors::ws;
use crossbeam::channel::{self, Sender, TryRecvError};
use directory_size::*;
use log::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
  path::PathBuf,
  sync::{Arc, Mutex},
  thread,
  time::Duration,
};

pub struct WebSocketActor {
  root_path: Vec<String>,
  current_dir: Vec<String>,
  thread_sender: Option<Sender<EventMessage>>,
}

impl WebSocketActor {
  pub fn new(root_path: &PathBuf) -> Self {
    Self {
      root_path: get_components(root_path),
      current_dir: Vec::new(),
      thread_sender: None,
    }
  }

  fn change_dir(&mut self, path: Vec<String>, ctx: &mut ws::WebsocketContext<Self>) {
    self.current_dir = path.clone();

    let full_path: PathBuf = self.root_path.iter().cloned().chain(path.clone()).collect();
    let entries = get_directory_entries(&full_path);

    let msg = EventMessage::DirectoryChange { path, entries };

    ctx
      .address()
      .do_send(TextMessage(serde_json::to_string(&msg).unwrap()));

    self.thread_sender.as_ref().unwrap().send(msg).unwrap();
  }
}

impl Actor for WebSocketActor {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    info!("ws started");

    let (thread_sender, thread_receiver) = channel::unbounded();
    self.thread_sender = Some(thread_sender);

    // send initial current directory entries
    self.change_dir(Vec::new(), ctx);


    let _thread = {
      let addr = ctx.address();
      let root_path: PathBuf = self.root_path.iter().collect();

      thread::spawn(move || {
        // get default current directory
        let (mut current_dir, mut subscribed_entries): (Vec<String>, Vec<String>) =
          match thread_receiver.recv().unwrap() {
            EventMessage::DirectoryChange { path, entries } => (
              path,
              entries
                .iter()
                .filter_map(|entry| {
                  if let Entry::Directory { name } = entry {
                    Some(name.to_owned())
                  } else {
                    None
                  }
                })
                .collect(),
            ),
            _ => unreachable!(),
          };

        current_dir = vec!["web_server".to_string()];

        let mut tree = Tree::new();

        let (_scanner, receiver) = FileSizeScanner::start(root_path);

        for file in receiver {
          let components = get_components(&file.0);
          tree.insert_file(file);

          if !components.starts_with(&current_dir) {
            // new file is not in current directory
            continue;
          }

          // current: ["web_server"]
          // new    : ["web_server", "mod.rs"]
          if (components.len() - 1) <= current_dir.len() {
            // skip file updates in current dir
            continue;
          }

          let relative_name = components[current_dir.len()].clone();
          let mut bap = current_dir.clone();
          bap.push(relative_name.clone());

          let size = tree.at(bap).unwrap().get_total_size();

          addr.do_send(TextMessage(
            serde_json::to_string(&EventMessage::SizeUpdate {
              name: relative_name,
              size,
            })
            .unwrap(),
          ));
        }
      })
    };

    // thread::spawn(move || {
    //   thread::sleep(Duration::from_millis(512));

    //   loop {
    //     // send chunk
    //     let mut chunk = Vec::new();
    //     let mut finished = false;
    //     let mut take_a_break = false;
    //     for _ in 0..10240 {
    //       // limit chunk size because it's hella big and makes chrome lag

    //       match receiver.try_recv() {
    //         Ok(file) => {
    //           chunk.push(FileSizeJson::from(file));
    //         }

    //         Err(e) => match e {
    //           TryRecvError::Empty => {
    //             take_a_break = true;
    //           }
    //           _ => {
    //             finished = true;
    //             break;
    //           }
    //         },
    //       }
    //     }

    //     info!("chunk with {}", chunk.len());
    //     addr.do_send(TextMessage(
    //       serde_json::to_string(&FileSizeStatusJson::Chunk(chunk)).unwrap(),
    //     ));

    //     if finished {
    //       break;
    //     }

    //     if take_a_break {
    //       thread::sleep(Duration::from_millis(512));
    //     }
    //   } // loop

    //   scanner.join();

    //   // send finish
    //   addr.do_send(TextMessage(
    //     serde_json::to_string(&FileSizeStatusJson::Finish).unwrap(),
    //   ));
    // });
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
            self.change_dir(path, ctx);
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
enum EventMessage {
  DirectoryChange {
    path: Vec<String>,
    entries: Vec<Entry>,
  },

  SizeUpdate {
    name: String,
    size: u64,
  },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
enum ControlMessage {
  ChangeDirectory { path: Vec<String> },
}
