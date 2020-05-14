use crate::websocket_handler::WebsocketHandler;
use failure::Error;
use futures::{channel::mpsc, prelude::*};
use log::{debug, info, warn};
use std::{net::SocketAddr, path::PathBuf, time::Duration};
use warp::{path::FullPath, Filter};

include!(concat!(env!("OUT_DIR"), "/parceljs.rs"));

pub async fn start(
  mut addr: SocketAddr,
  root_path: PathBuf,
  keep_open: bool,
  no_browser: bool,
) -> Result<(), Error> {
  info!("starting http/websocket server");

  let mut tries: u8 = 0;
  loop {
    let root_path = root_path.clone();
    let ok = _start(addr, root_path, keep_open);

    match ok {
      Ok((addr, fut)) => {
        info!("listening on http://{}/", addr);

        if !no_browser {
          tokio::spawn(async move {
            tokio::time::delay_for(Duration::from_millis(100)).await;

            if let Err(err) = open::that(format!("http://{}/", addr)) {
              warn!("couldn't open http link: {}", err);
            }
          });
        }

        fut.await;

        return Ok(());
      }

      Err(err) => {
        if tries <= 4 {
          tries += 1;
          let new_port = addr.port() + 1;
          warn!("{}, trying next port {}", err, new_port);
          addr.set_port(new_port);
          continue;
        }
        return Err(err.into());
      }
    }
  }
}

fn _start(
  addr: SocketAddr,
  root_path: PathBuf,
  keep_open: bool,
) -> Result<(SocketAddr, impl Future<Output = ()> + 'static), warp::Error> {
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
      PARCELJS.as_reply(path)
    }));

  warp::serve(routes).try_bind_with_graceful_shutdown(addr, async move {
    shutdown_receiver.next().await;
  })
}
