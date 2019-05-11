use actix::prelude::*;

#[derive(Debug, Message)]
pub struct BinaryMessage(pub Vec<u8>);

impl From<Vec<u8>> for BinaryMessage {
  fn from(s: Vec<u8>) -> Self {
    Self(s)
  }
}

impl From<&[u8]> for BinaryMessage {
  fn from(s: &[u8]) -> Self {
    Self(s.to_vec())
  }
}
