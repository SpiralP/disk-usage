mod string_message;

use self::string_message::StringMessage;
use actix::prelude::*;
use actix_web::ws;
use log::*;
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;

pub struct WebSocketActor {}

impl WebSocketActor {
  pub fn new() -> Self {
    Self {}
  }
}

impl Actor for WebSocketActor {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    info!("ws started");
  }

  fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
    info!("ws stopping");

    // for (_id, worker) in self.workers.drain() {
    //   worker.do_send(Stop);
    // }

    Running::Stop
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {
    info!("ws stopped");
  }
}

impl Handler<StringMessage> for WebSocketActor {
  type Result = ();

  fn handle(&mut self, msg: StringMessage, ctx: &mut Self::Context) -> Self::Result {
    ctx.text(msg.0);
  }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for WebSocketActor {
  fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
    match msg {
      ws::Message::Text(text) => {
        // let msg: WorkerSendMessage = serde_json::from_str(&text).unwrap();
        println!("{}", text);
      }
      ws::Message::Close(_) => ctx.stop(),
      _ => {}
    }
  }
}

// // {state: "start", module: "test"}
// #[derive(Deserialize)]
// #[serde(rename_all = "camelCase")]
// #[serde(tag = "state")]
// enum WorkerSendMessage {
//   Start { r#type: ModuleType, data: String },
//   Stop { id: u8 },
// }
