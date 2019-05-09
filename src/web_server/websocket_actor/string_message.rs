use actix::prelude::*;

#[derive(Debug, Message)]
pub struct StringMessage(pub String);

impl From<String> for StringMessage {
  fn from(s: String) -> Self {
    Self(s)
  }
}

impl From<&str> for StringMessage {
  fn from(s: &str) -> Self {
    Self(s.to_string())
  }
}
