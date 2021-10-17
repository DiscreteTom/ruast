use async_trait::async_trait;
use bytes::Bytes;
use std::{collections::HashMap, fmt};
use tokio::sync::mpsc::Sender;

use crate::controller::server::ServerManager;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type MultiResult<T> = HashMap<u32, Result<T>>;

#[async_trait]
pub trait Peer {
  fn id(&self) -> u32;
  fn set_tag(&mut self, tag: String);
  fn tag(&self) -> &str;
  async fn write(&mut self, data: Bytes) -> Result<()>;
  fn stop(&mut self);
}

#[async_trait]
pub trait PeerBuilder {
  fn id(&mut self, id: u32) -> &mut dyn PeerBuilder;
  fn tag(&mut self, tag: String) -> &mut dyn PeerBuilder;
  fn server_tx(&mut self, server_tx: Sender<ServerEvent>) -> &mut dyn PeerBuilder;
  async fn build(&mut self) -> Result<Box<dyn Peer + Send>>;

  fn get_id(&self) -> Option<u32>;
  fn get_tag(&self) -> &str;
}

#[derive(Clone, Debug)]
pub struct PeerMsg {
  pub peer_id: u32,
  pub data: Bytes,
}

#[derive(Debug)]
pub enum ServerEvent {
  Custom(u32),
  PeerMsg(PeerMsg),
  RemovePeer(u32),
}

pub trait Plugin {
  fn start(&self, code: u32, server_tx: Sender<ServerEvent>);
  fn handle(&self, s: &mut ServerManager);
}

pub trait PeerIdAllocator {
  fn allocate(&mut self, peer: &Box<dyn PeerBuilder>) -> u32;
}

#[derive(Debug)]
pub enum Error {
  PeerNotExist(u32),
  PeerAlreadyExist(u32),
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
