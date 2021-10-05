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
  Custom(u32),
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
