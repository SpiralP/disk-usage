mod binary_message;
mod dir;
mod text_message;
mod tree;

use self::{binary_message::*, dir::*, text_message::*, tree::*};
use super::*;
use actix::prelude::*;
use actix_web_actors::ws;
use crossbeam::channel::TryRecvError;
use directory_size::*;
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
    println!("ws started");

    // send start
    ctx.address().do_send(TextMessage(
      serde_json::to_string(&FileSizeStatusJson::Start).unwrap(),
    ));

    let root_path = self.root_path.clone();
    let ctx = ctx.address();

    let (scanner, receiver) = FileSizeScanner::start(root_path);

    thread::spawn(move || {
      thread::sleep(Duration::from_millis(512));


      loop {
        // send chunk
        let mut chunk = Vec::new();
        let mut finished = false;
        let mut take_a_break = false;
        for _ in 0..10240 {
          // limit chunk size because it's hella big and makes chrome lag

          match receiver.try_recv() {
            Ok(file) => {
              chunk.push(FileSizeJson::from(file));
            }

            Err(e) => match e {
              TryRecvError::Empty => {
                take_a_break = true;
              }
              _ => {
                finished = true;
                break;
              }
            },
          }
        }

        println!("chunk with {}", chunk.len());
        ctx.do_send(TextMessage(
          serde_json::to_string(&FileSizeStatusJson::Chunk(chunk)).unwrap(),
        ));

        if finished {
          break;
        }

        if take_a_break {
          thread::sleep(Duration::from_millis(512));
        }
      } // loop

      scanner.join();

      // send finish
      ctx.do_send(TextMessage(
        serde_json::to_string(&FileSizeStatusJson::Finish).unwrap(),
      ));
    });
  }

  fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
    println!("ws stopping");

    Running::Stop
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {
    println!("ws stopped");
  }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "t", content = "c")]
enum FileSizeStatusJson {
  Start,
  Chunk(Vec<FileSizeJson>),
  Finish,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileSizeJson(Vec<String>, u64);
impl From<FileSize> for FileSizeJson {
  fn from(o: FileSize) -> Self {
    Self(
      o.0
        .iter()
        .map(|os_str| os_str.to_string_lossy().to_string())
        .collect(),
      o.1,
    )
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
        println!("got {}", text);
      }
      ws::Message::Close(_) => ctx.stop(),
      _ => {
        println!("other kind! {:#?}", msg);
      }
    }
  }
}
