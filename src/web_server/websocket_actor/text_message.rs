use actix::prelude::*;

#[derive(Debug, Message)]
pub struct TextMessage(pub String);

impl From<String> for TextMessage {
  fn from(s: String) -> Self {
    Self(s)
  }
}

impl From<&str> for TextMessage {
  fn from(s: &str) -> Self {
    Self(s.to_string())
  }
}
