mod binary_message;
mod dir;
mod text_message;
mod tree;

use self::{binary_message::*, dir::*, text_message::*, tree::*};
use actix::prelude::*;
use actix_web_actors::ws;
use crossbeam::channel::TryRecvError;
use directory_size::*;
use log::*;
use serde::Serialize;
use serde_json;
use std::{path::PathBuf, thread, time::Duration};

pub struct WebSocketActor {
  root_path: PathBuf,
}

impl WebSocketActor {
  pub fn new(root_path: String) -> Self {
    Self {
      root_path: root_path.parse().unwrap(),
    }
  }
}

impl Actor for WebSocketActor {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    info!("ws started");

    let addr = ctx.address();

    // send initial current directory entries
    let entries = get_directory_entries(&self.root_path);
    addr.do_send(TextMessage(
      serde_json::to_string(&WebSocketMessage::DirectoryChange { entries }).unwrap(),
    ));


    let (_scanner, receiver) = FileSizeScanner::start(self.root_path.clone());

    let current_dir: Vec<String> = get_components("src".parse().unwrap());

    thread::spawn(move || {
      let mut tree = Tree::new();

      for file in receiver {
        let components = get_components(file.0.clone());
        tree.insert_file(file);

        let mut ok = true;
        for (i, dir) in current_dir.iter().enumerate() {
          if *dir != components[i] {
            ok = false;
            break;
          }
        }

        // current_dir: ["src"]
        // components:  ["src", "web_server", "bap.rs"]
        if ok && (components.len() - 1) > current_dir.len() {
          let relative_name = components[current_dir.len()].clone();
          let mut bap = current_dir.clone();
          bap.push(relative_name.clone());

          addr.do_send(TextMessage(
            serde_json::to_string(&WebSocketMessage::SizeUpdate {
              name: relative_name.clone(),
              size: tree.at(bap).unwrap().get_total_size(),
            })
            .unwrap(),
          ));
        }
      }
    });

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
        info!("change: {}", text);
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
enum WebSocketMessage {
  DirectoryChange { entries: Vec<Entry> },
  SizeUpdate { name: String, size: u64 },
}
