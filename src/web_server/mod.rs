#![allow(clippy::unreadable_literal)]

mod static_files_includedir;
mod websocket_actor;

use self::{static_files_includedir::*, websocket_actor::*};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use std::path::PathBuf;

include!(concat!(env!("OUT_DIR"), "/web_files.rs"));


pub fn start_web_server(root_path: PathBuf) {
  HttpServer::new(move || {
    let root_path = root_path.clone();

    App::new()
      .data(MyData {
        base_path: "dist",
        files: &WEB_FILES,
      })
      .service(
        web::resource("/ws")
          .default_service(web::route().to(HttpResponse::MethodNotAllowed))
          .route(
            web::get().to(move |req: HttpRequest, stream: web::Payload| {

              ws::start(WebSocketActor::new(&root_path), &req, stream)
            }),
          ),
      )
      .service(static_files_service)
  })
  .bind("127.0.0.1:8080")
  .unwrap()
  .run()
  .unwrap();
}
