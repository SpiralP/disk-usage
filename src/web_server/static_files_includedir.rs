use actix::prelude::*;
use actix_web::{
  get, middleware,
  web::{self, Path},
  App, Error, HttpRequest, HttpResponse, HttpServer,
};
use includedir;

pub struct StaticFilesIncludedir {
  base_path: &'static str,
  files: &'static includedir::Files,
}

impl StaticFilesIncludedir {
  pub fn new(files: &'static includedir::Files, base_path: &'static str) -> Self {
    Self { files, base_path }
  }
}

// impl Handler<S> for StaticFilesIncludedir {
//   type Result = Result<HttpResponse, Error>;

//   fn handle(&self, req: &HttpRequest) -> Self::Result {
//     match *req.method() {
//       actix_web::http::Method::GET => {}

//       _ => {
//         return Ok(HttpResponse::MethodNotAllowed().finish());
//       }
//     }

//     let file_path = match req.path() {
//       "/" => "/index.html",
//       other => other,
//     };

//     match self.files.get(&format!("{}{}", self.base_path, file_path)) {
//       Ok(bytes) => Ok(HttpResponse::Ok().body(bytes)),
//       Err(_) => Ok(HttpResponse::NotFound().finish()),
//     }
//   }
// }

pub struct MyData {
  pub base_path: &'static str,
  pub files: &'static includedir::Files,
}

#[get("/*")]
pub fn static_files_service(req: HttpRequest) -> HttpResponse {
  let file_path = match req.path() {
    "/" => "/index.html",
    other => other,
  };

  let data: web::Data<MyData> = req.get_app_data().unwrap();

  match data.files.get(&format!("{}{}", data.base_path, file_path)) {
    Ok(bytes) => HttpResponse::Ok().body(bytes.into_owned()),
    Err(_) => HttpResponse::NotFound().finish(),
  }
}

// #[get("/*")]
// fn static_files_service(req: HttpRequest) -> String {
//   println!("REQ: {:?}", req);

//   String::new()
// }
