use actix_web::{http::header::ContentType, web, HttpRequest, HttpResponse, Route};
use includedir;

pub fn static_files_route(base_path: &'static str, files: &'static includedir::Files) -> Route {
  web::get().to(move |req: HttpRequest| -> HttpResponse {
    let file_path = match req.path() {
      "/" => "/index.html",
      other => other,
    };

    match files.get(&format!("{}{}", base_path, file_path)) {
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
  })
}
