mod string_message;

use actix::prelude::*;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;


pub struct WebsocketActor;

impl Actor for WebsocketActor {
  type Context = ws::WebsocketContext<Self>;
}

// Handler for ws::Message messages
impl StreamHandler<ws::Message, ws::ProtocolError> for WebsocketActor {
  fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
    match msg {
      ws::Message::Ping(msg) => ctx.pong(&msg),
      ws::Message::Text(text) => ctx.text(text),
      ws::Message::Binary(bin) => ctx.binary(bin),
      _ => (),
    }
  }
}
