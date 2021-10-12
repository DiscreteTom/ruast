use bytes::Bytes;
use std::{collections::HashMap, fmt};
use tokio::sync::mpsc::Sender;

use crate::controller::EventHub;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type MultiResult<T> = HashMap<u32, Result<T>>;

pub trait Peer {
  fn tx(&self) -> &Sender<PeerEvent>;
  fn id(&self) -> u32;
  fn set_tag(&mut self, tag: String);
  fn tag(&self) -> &str;
}

pub trait PeerBuilder {
  fn id(self, id: u32) -> Box<dyn PeerBuilder>;
  fn tag(self, tag: String) -> Box<dyn PeerBuilder>;
  fn buffer(self, buffer: usize) -> Box<dyn PeerBuilder>;
  fn hub_tx(self, hub_tx: Sender<HubEvent>) -> Box<dyn PeerBuilder>;
  fn build(self) -> Result<Box<dyn Peer>>;

  fn get_id(&self) -> Option<u32>;
  fn get_tag(&self) -> &str;
}

#[derive(Clone, Debug)]
pub struct PeerMsg {
  pub peer_id: u32,
  pub data: Bytes,
}

#[derive(Debug)]
pub enum HubEvent {
  Custom(u32),
  PeerMsg(PeerMsg),
  RemovePeer(u32),
  Stop,
}

#[derive(Debug)]
pub enum PeerEvent {
  Write(Bytes),
  Stop,
}

pub trait Plugin {
  fn start(&self);
  fn handle(&self, hub: &EventHub);
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
