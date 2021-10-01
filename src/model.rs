use std::{
  error::Error,
  fmt,
  sync::Weak,
  sync::{Arc, Mutex},
  time::SystemTime,
};

pub trait Peer: Send + Sync {
  fn write(&mut self, data: Arc<Vec<u8>>) -> Result<(), Box<dyn Error>>;
  fn id(&self) -> &str;
  fn set_tag(&mut self, tag: &str);
  fn tag(&self) -> &str;
}

pub struct PeerMsg {
  pub peer: Weak<Mutex<dyn Peer>>,
  pub data: Arc<Vec<u8>>,
  pub time: SystemTime,
}

pub enum ServerEvent {
  Msg(PeerMsg),
  Stop,
}

#[derive(Debug)]
pub enum ServerError {
  PeerNotExist(String),
  PeerAlreadyExist(String),
}

impl fmt::Display for ServerError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ServerError::PeerNotExist(id) => write!(f, "peer not exist, id={}", id),
      ServerError::PeerAlreadyExist(id) => write!(f, "peer already exist, id={}", id),
    }
  }
}

impl Error for ServerError {}

pub trait GameServer {
  fn add_peer(&self, peer: Arc<Mutex<dyn Peer>>) -> Result<(), Box<dyn Error>>;
  fn remove_peer(&self, id: &str) -> Result<(), Box<dyn Error>>;
  fn stop(&self);
  fn for_each_peer(&self, f: fn(Arc<Mutex<dyn Peer>>));
  fn apply_to(&self, id: &str, f: fn(Arc<Mutex<dyn Peer>>)) -> Result<(), Box<dyn Error>>;
}
