use std::{collections::HashMap, error::Error, fmt, sync::Arc, time::SystemTime};

pub trait Peer {
  fn write(&mut self, data: Arc<Vec<u8>>) -> Result<(), Box<dyn Error>>;
  fn id(&self) -> i32;
  fn set_tag(&mut self, tag: &str);
  fn tag(&self) -> &str;
  fn start(&mut self) -> Result<(), Box<dyn Error>>;
}

pub struct PeerMsg {
  pub peer_id: i32,
  pub data: Arc<Vec<u8>>,
  pub time: SystemTime,
}

pub enum ServerEvent {
  PeerMsg(PeerMsg),
  Stop,
}

#[derive(Debug)]
pub enum ServerError {
  PeerNotExist(i32),
  PeerAlreadyExist(i32),
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
  fn add_peer(&self, peer: Box<dyn Peer>) -> Result<(), Box<dyn Error>>;
  fn remove_peer(&self, id: i32) -> Result<(), Box<dyn Error>>;
  fn stop(&self);

  fn for_each_peer<F, T>(&self, f: F) -> HashMap<i32, Result<T, Box<dyn Error>>>
  where
    F: Fn(&mut Box<dyn Peer>) -> Result<T, Box<dyn Error>>;

  fn apply_to<F, T>(&self, id: i32, f: F) -> Result<T, Box<dyn Error>>
  where
    F: FnOnce(&mut Box<dyn Peer>) -> Result<T, Box<dyn Error>>;

  fn write_to(&self, id: i32, data: Arc<Vec<u8>>) -> Result<(), Box<dyn Error>> {
    self.apply_to(id, |p| p.write(data))
  }
  fn broadcast<F>(
    &self,
    data: Arc<Vec<u8>>,
    selector: F,
  ) -> HashMap<i32, Result<bool, Box<dyn Error>>>
  where
    F: Fn(&Box<dyn Peer>) -> bool,
  {
    self.for_each_peer(|p| {
      if selector(p) {
        match p.write(data.clone()) {
          Ok(_) => Ok(true),
          Err(e) => Err(e),
        }
      } else {
        Ok(false)
      }
    })
  }
}
