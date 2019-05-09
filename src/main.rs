#![allow(clippy::unreadable_literal)]

// react/tsx
// POST /delete or even DELETE
// multi thread scanning, live view

mod static_files_includedir;

use actix_web::{http, server, App, Path, Responder};
use static_files_includedir::StaticFilesIncludedir;

include!(concat!(env!("OUT_DIR"), "/web_files.rs"));


fn index(info: Path<(u32, String)>) -> impl Responder {
  format!("Hello {}! id:{}", info.1, info.0)
}

fn main() {
  server::new(|| {
    App::new()
      .route("/{id}/{name}/index.html", http::Method::GET, index)
      .handler("/", StaticFilesIncludedir::new(&WEB_FILES, "dist"))
  })
  .bind("127.0.0.1:8080")
  .unwrap()
  .run();
}
