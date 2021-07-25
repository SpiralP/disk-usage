#![warn(clippy::pedantic)]

mod logger;
mod web_server;
mod websocket_handler;

use clap::{clap_app, crate_name, crate_version};
use failure::Error;
use log::warn;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

const IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const PORT: u16 = 8000;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let ip = IP;
    let port = PORT;

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
    let keep_open = matches.is_present("keep_open");
    let path: PathBuf = matches.value_of("path").unwrap().into();

    web_server::start(SocketAddr::new(ip, port), path, keep_open, no_browser).await?;

    Ok(())
}
