mod static_files_includedir;
mod websocket_actor;

use self::{static_files_includedir::*, websocket_actor::*};
use actix_web::{web, App, HttpRequest, HttpServer};
use actix_web_actors::ws;
use open;
use std::path::PathBuf;

const LISTEN_ADDR: &str = "127.0.0.1:8181";

pub fn start(root_path: PathBuf) {
  if let Err(err) = open::that(format!("http://{}/", LISTEN_ADDR)) {
    eprintln!("couldn't open http link: {}", err);
  }

  HttpServer::new(move || {
    let root_path = root_path.clone();

    App::new()
      .service(
        web::resource("/ws").to(move |req: HttpRequest, stream: web::Payload| {
          ws::start(WebSocketActor::new(&root_path), &req, stream)
        }),
      )
      .route("/*", static_files_route())
  })
  .bind(LISTEN_ADDR)
  .unwrap()
  .run()
  .unwrap();
}
