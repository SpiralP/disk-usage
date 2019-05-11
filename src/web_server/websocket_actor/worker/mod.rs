use super::*;
use actix::prelude::*;
use crossbeam::channel::TryRecvError;
use directory_size::*;
use serde::Serialize;
use serde_json;
use std::{path::PathBuf, thread, time::Duration};

pub struct Worker {
  ws: Addr<WebSocketActor>,
  root_path: PathBuf,
}


impl Worker {
  pub fn start(ws: Addr<WebSocketActor>, root_path: String) -> Addr<Self> {
    let worker = Worker {
      ws,
      root_path: root_path.parse().unwrap(),
    };
    worker.start()
  }
}

impl Actor for Worker {
  type Context = Context<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    println!("worker started");

    // send start
    self.ws.do_send(TextMessage(
      serde_json::to_string(&FileSizeStatusJson::Start).unwrap(),
    ));

    let root_path = self.root_path.clone();
    let ws = self.ws.clone();
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
        ws.do_send(TextMessage(
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
      ws.do_send(TextMessage(
        serde_json::to_string(&FileSizeStatusJson::Finish).unwrap(),
      ));


      ctx.do_send(Stop);
    });
  }

  fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
    println!("worker stopping");

    Running::Stop
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {
    println!("worker stopped");
  }
}

#[derive(Debug, Message)]
pub struct Stop;

impl Handler<Stop> for Worker {
  type Result = ();

  fn handle(&mut self, _: Stop, ctx: &mut Self::Context) {
    ctx.stop();
  }
}


// impl StreamHandler<Message, ()> for Worker {
//   fn handle(&mut self, msg: Message, _ctx: &mut Self::Context) {
//     self.ws.do_send(msg); // spawn dies if stream dies!
//   }
// }


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
struct FileSizeJson(PathBuf, u64);
impl From<FileSize> for FileSizeJson {
  fn from(o: FileSize) -> Self {
    Self(o.0, o.1)
  }
}
