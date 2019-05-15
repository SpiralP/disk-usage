// react/tsx
// POST /delete or even DELETE
// multi thread scanning, live view

mod logger;
mod web_server;

use self::{logger::initialize_logger, web_server::start_web_server};

fn main() {
  initialize_logger(false);

  start_web_server(".".parse().unwrap());
}
