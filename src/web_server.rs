use crate::websocket_handler::WebsocketHandler;
use log::{debug, info};
use parceljs::warp::ParceljsResponder;
use std::{net::SocketAddr, path::PathBuf};
use warp::{path::FullPath, Filter};

pub async fn start(addr: SocketAddr, root_path: PathBuf) {
  info!("starting http/websocket server on http://{}/", addr);

  let routes = warp::path("ws")
    .and(warp::ws())
    .map(move |ws: warp::ws::Ws| {
      let root_path = root_path.clone();
      ws.on_upgrade(move |ws| {
        async move {
          debug!("websocket connection");

          let mut handler = WebsocketHandler::new(&root_path);
          handler.start(ws).await.unwrap();

          // we don't want to use tokio here because iterator streams
          // block the other http request futures by taking from the pool
          // spawn("websocket future thread", move || {
          //   block_on(async {
          //     websocket::start(ws, &root_path).await;
          //   })
          //   .expect("block_on");
          // });
        }
      })
    })
    .or(warp::path::full().map(|path: FullPath| {
      debug!("http {}", path.as_str());
      ParceljsResponder::new(path)
    }));

  warp::serve(routes).bind(addr).await;
}
