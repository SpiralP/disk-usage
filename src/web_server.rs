use crate::websocket_handler::WebsocketHandler;
use futures::{channel::mpsc, prelude::*};
use log::{debug, info};
use parceljs::warp::ParceljsResponder;
use std::{net::SocketAddr, path::PathBuf};
use warp::{path::FullPath, Filter};

pub async fn start(addr: SocketAddr, root_path: PathBuf, keep_open: bool) {
  info!("starting http/websocket server");

  let (shutdown_sender, mut shutdown_receiver) = mpsc::channel(1);

  let routes = warp::path("ws")
    .and(warp::ws())
    .map(move |ws: warp::ws::Ws| {
      let root_path = root_path.clone();
      let mut shutdown_sender = shutdown_sender.clone();

      ws.on_upgrade(move |ws| async move {
        debug!("websocket upgraded");

        {
          WebsocketHandler::run(&root_path, ws).await;
        }

        info!("ws stopped");

        if !keep_open {
          shutdown_sender.send(()).await.unwrap();
        }
      })
    })
    .or(warp::path::full().map(|path: FullPath| {
      debug!("http {}", path.as_str());
      ParceljsResponder::new(path)
    }));

  let (addr, fut) = warp::serve(routes).bind_with_graceful_shutdown(addr, async move {
    shutdown_receiver.next().await;
  });

  info!("listening on http://{}/", addr);

  fut.await;
}
