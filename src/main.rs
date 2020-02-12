#![warn(clippy::pedantic)]

mod error;
mod logger;
mod web_server;
mod websocket_handler;

use crate::error::*;
use clap::{clap_app, crate_name, crate_version};
use log::warn;
use std::{
  net::{IpAddr, Ipv4Addr, SocketAddr},
  path::PathBuf,
  time::Duration,
};

#[tokio::main]
async fn main() -> Result<()> {
  let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
  let web_server_addr = SocketAddr::new(ip, 8181);

  let matches = clap_app!(app =>
      (name: crate_name!())
      (version: crate_version!())

      (@arg debug: -v --verbose --debug ... "Show debug messages, multiple flags for higher verbosity")
      (@arg keep_open: -k "Keep program alive after websocket closed")
      (@arg no_browser: -n --("no-browser") "Don't open browser")

      (@arg path: [PATH] +required default_value(".") "Path")
  )
  .get_matches();

  #[cfg(debug_assertions)]
  logger::initialize(true, false);

  #[cfg(not(debug_assertions))]
  logger::initialize(
    matches.is_present("debug"),
    matches.occurrences_of("debug") > 1,
  );

  let no_browser = matches.is_present("no_browser");

  if !no_browser {
    tokio::spawn(async move {
      tokio::time::delay_for(Duration::from_millis(100)).await;

      if let Err(err) = open::that(format!("http://{}/", web_server_addr)) {
        warn!("couldn't open http link: {}", err);
      }
    });
  }

  let path: PathBuf = matches.value_of("path").unwrap().into();

  let keep_open = matches.is_present("keep_open");
  web_server::start(web_server_addr, path, keep_open).await;

  Ok(())
}
