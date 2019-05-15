// react/tsx
// POST /delete or even DELETE
// multi thread scanning, live view

mod logger;
mod web_server;

use self::{logger::initialize_logger, web_server::start_web_server};
use std::env::args;

fn main() {
  initialize_logger(false);

  start_web_server(
    args()
      .nth(1)
      .unwrap_or_else(|| ".".to_string())
      .parse()
      .unwrap(),
  );
}
