use std::{
  error::Error,
  fmt,
  sync::Arc,
  sync::{mpsc::Sender, Weak},
  time::SystemTime,
};

pub trait Peer: Send + Sync {
  fn write(&mut self, data: Arc<Vec<u8>>) -> Result<(), Box<dyn Error>>;
  fn id(&self) -> i32;
  fn set_tag(&mut self, tag: &str);
  fn tag(&self) -> &str;
}

pub struct PeerMsg {
  pub peer: Weak<dyn Peer>,
  pub data: Arc<Vec<u8>>,
  pub time: SystemTime,
}

pub enum ServerEvent {
  Msg(PeerMsg),
  Stop,
}

#[derive(Debug)]
pub enum ServerError {
  PeerNotExist(i32),
}

impl fmt::Display for ServerError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      ServerError::PeerNotExist(id) => write!(f, "peer not exist, id={}", id),
    }
  }
}

impl Error for ServerError {}

pub trait GameServer {
  fn new_peer<F>(&self, generator: F) -> Result<i32, Box<dyn Error>>
  where
    F: Fn(i32, Sender<ServerEvent>) -> Result<Arc<dyn Peer>, Box<dyn Error>>;
  fn remove_peer(&self, id: i32) -> Result<(), Box<dyn Error>>;
  fn stop(&self);
  fn for_each_peer(&self, f: Box<dyn FnMut(&Arc<dyn Peer>)>);
  fn apply_to(&self, id: i32, f: Box<dyn FnMut(&Arc<dyn Peer>)>) -> Result<(), Box<dyn Error>>;
}
