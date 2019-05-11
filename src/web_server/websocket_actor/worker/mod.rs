use super::*;
use actix::prelude::*;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use directory_size::*;
use serde::Serialize;
use serde_json;
use std::{path::PathBuf, thread};

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
    // self.ws.do_send(Message(
    //   // TODO make a message type for json stuff
    //   serde_json::to_string(&WorkerStartMessage {
    //     id: format!("{}", self.id),
    //     state: WorkerState::Start,
    //     r#type: self.module.as_ref().unwrap().get_type(),
    //   })
    //   .unwrap(),
    // ));

    let ws = self.ws.clone();
    let root_path = self.root_path.clone();
    thread::spawn(move || {
      let (scanner, receiver) = FileSizeScanner::start(root_path);

      // TODO maybe buffer many messages in chunks
      for file in receiver {
        ws.do_send(StringMessage(
          serde_json::to_string(&FileSizeJson::from(file)).unwrap(),
        ));
      }
      println!("done sending all");

      scanner.join();
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
struct FileSizeJson(PathBuf, u64);
impl From<FileSize> for FileSizeJson {
  fn from(o: FileSize) -> Self {
    Self(o.0, o.1)
  }
}
