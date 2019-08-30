use actix_web::{http::header::ContentType, web, HttpRequest, HttpResponse, Route};
use parceljs;

pub fn static_files_route() -> Route {
  web::get().to(move |req: HttpRequest| -> HttpResponse {
    let file_path = req.path();

    match parceljs::get_file(file_path) {
      Ok(bytes) => {
        let mut builder = HttpResponse::Ok();

        if let Some(content_type) = parceljs::get_content_type(&file_path) {
          builder.set(ContentType(content_type.parse().unwrap()));
        }

        builder.body(bytes.into_owned())
      }
      Err(_) => HttpResponse::NotFound().finish(),
    }
  })
}
