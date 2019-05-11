mod binary_message;
mod text_message;
mod worker;

use self::{binary_message::*, text_message::*, worker::*};
use actix::prelude::*;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;


pub struct WebSocketActor {
  worker: Option<Addr<Worker>>,
}
impl WebSocketActor {
  pub fn new() -> Self {
    Self { worker: None }
  }
}

impl Actor for WebSocketActor {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    println!("ws started");
  }

  fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
    println!("ws stopping");

    if let Some(worker) = self.worker.take() {
      worker.do_send(Stop);
    }

    Running::Stop
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {
    println!("ws stopped");
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
        self.worker = Some(Worker::start(ctx.address(), text));
      }
      ws::Message::Close(_) => ctx.stop(),
      _ => {
        println!("other kind! {:#?}", msg);
      }
    }
  }
}
