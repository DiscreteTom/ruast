use std::{collections::HashMap, error::Error, fmt, sync::Arc, time::SystemTime};

pub type Data = Arc<Vec<u8>>;
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub type MultiResult<T> = HashMap<i32, Result<T>>;

pub trait Peer {
  fn write(&mut self, data: Data) -> Result<()>;
  fn id(&self) -> i32;
  fn set_tag(&mut self, tag: &str);
  fn tag(&self) -> &str;
  fn start(&mut self) -> Result<()> {
    Ok(())
  }
}

pub struct PeerMsg {
  pub peer_id: i32,
  pub data: Data,
  pub time: SystemTime,
}

impl Clone for PeerMsg {
  fn clone(&self) -> Self {
    Self {
      peer_id: self.peer_id.clone(),
      data: self.data.clone(),
      time: self.time.clone(),
    }
  }
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
  fn add_peer(&self, peer: Box<dyn Peer>) -> Result<()>;
  fn remove_peer(&self, id: i32) -> Result<()>;
  fn stop(&self);

  fn for_each_peer<F, T>(&self, f: F) -> MultiResult<T>
  where
    F: Fn(&mut Box<dyn Peer>) -> Result<T>;

  fn apply_to<F, T>(&self, id: i32, f: F) -> Result<T>
  where
    F: FnOnce(&mut Box<dyn Peer>) -> Result<T>;

  fn write_to(&self, id: i32, data: Data) -> Result<()> {
    self.apply_to(id, |p| p.write(data))
  }
  fn echo(&self, msg: PeerMsg) -> Result<()> {
    self.write_to(msg.peer_id, msg.data)
  }
  fn broadcast<F>(&self, data: Data, selector: F) -> MultiResult<bool>
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
  fn broadcast_all(&self, data: Data) -> MultiResult<bool> {
    self.broadcast(data, |_| true)
  }
}
