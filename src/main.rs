#![warn(clippy::pedantic)]

mod logger;
mod web_server;

use std::env::args;

fn main() {
  logger::initialize(false);

  web_server::start(
    args()
      .nth(1)
      .unwrap_or_else(|| ".".to_string())
      .parse()
      .unwrap(),
  );
}
