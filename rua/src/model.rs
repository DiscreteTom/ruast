use bytes::Bytes;
use std::{collections::HashMap, fmt};
use tokio::sync::mpsc::Sender;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type MultiResult<T> = HashMap<i32, Result<T>>;

pub trait Peer {
  fn tx(&self) -> &Sender<Bytes>;
  fn id(&self) -> i32;
  fn set_tag(&mut self, tag: &str);
  fn tag(&self) -> &str;
  fn start(&mut self) -> Result<()> {
    Ok(())
  }
}

#[derive(Clone, Debug)]
pub struct PeerMsg {
  pub peer_id: i32,
  pub data: Bytes,
}

#[derive(Debug)]
pub enum HubEvent {
  Custom(u32),
  PeerMsg(PeerMsg),
  Stop,
}

#[derive(Debug)]
pub enum PeerEvent {
  Write(Bytes),
  Stop,
}

#[derive(Debug)]
pub enum Error {
  PeerNotExist(i32),
  PeerAlreadyExist(i32),
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::PeerNotExist(id) => write!(f, "peer not exist, id={}", id),
      Error::PeerAlreadyExist(id) => write!(f, "peer already exist, id={}", id),
    }
  }
}

impl std::error::Error for Error {}
