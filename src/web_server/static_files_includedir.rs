use actix_web;
use actix_web::dev::*;
use actix_web::{Error, HttpRequest, HttpResponse};
use includedir;

pub struct StaticFilesIncludedir {
  base_path: &'static str,
  files: &'static includedir::Files,
}

impl<S: 'static> Handler<S> for StaticFilesIncludedir {
  type Result = Result<HttpResponse, Error>;

  fn handle(&self, req: &HttpRequest<S>) -> Self::Result {
    match *req.method() {
      actix_web::http::Method::GET => {}

      _ => {
        return Ok(HttpResponse::MethodNotAllowed().finish());
      }
    }

    let file_path = match req.path() {
      "/" => "/index.html",
      other => other,
    };

    match self.files.get(&format!("{}{}", self.base_path, file_path)) {
      Ok(bytes) => Ok(HttpResponse::Ok().body(bytes)),
      Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
  }
}

impl StaticFilesIncludedir {
  pub fn new(files: &'static includedir::Files, base_path: &'static str) -> Self {
    Self { files, base_path }
  }
}
