#![allow(clippy::unreadable_literal)]

mod static_files_includedir;
mod websocket_actor;

use self::{static_files_includedir::StaticFilesIncludedir, websocket_actor::*};
use actix_web::{http, server, ws, App, Path, Responder};

include!(concat!(env!("OUT_DIR"), "/web_files.rs"));


fn index(info: Path<(u32, String)>) -> impl Responder {
  format!("Hello {}! id:{}", info.1, info.0)
}

pub fn start_web_server() {
  server::new(|| {
    App::new()
      .route("/{id}/{name}/index.html", http::Method::GET, index)
      .resource("/ws", |r| {
        r.method(http::Method::GET)
          .f(|request| ws::start(request, WebSocketActor::new()))
      })
      .handler("/", StaticFilesIncludedir::new(&WEB_FILES, "dist"))
  })
  .bind("127.0.0.1:8080")
  .unwrap()
  .run();
}
