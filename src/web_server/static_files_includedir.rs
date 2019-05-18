use actix_web::{get, http::header::ContentType, web, HttpRequest, HttpResponse};
use includedir;

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
    Ok(bytes) => {
      let mut builder = HttpResponse::Ok();

      if file_path.ends_with(".css") {
        builder.set(ContentType("text/css; charset=utf-8".parse().unwrap()));
      } else if file_path.ends_with(".js") {
        builder.set(ContentType(
          "application/javascript; charset=utf-8".parse().unwrap(),
        ));
      } else if file_path.ends_with(".html") {
        builder.set(ContentType("text/html; charset=utf-8".parse().unwrap()));
      }

      builder.body(bytes.into_owned())
    }
    Err(_) => HttpResponse::NotFound().finish(),
  }
}

// #[get("/*")]
// fn static_files_service(req: HttpRequest) -> String {
//   println!("REQ: {:?}", req);

//   String::new()
// }
