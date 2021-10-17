use bytes::Bytes;
use std::collections::{hash_map::Entry, HashMap};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::model::{Error, HubEvent, MultiResult, Peer, PeerMsg, Result};

/// Use `EventHub::new()` or `EventHub::with_tx()` to create event hub.
pub struct EventHub {
  peers: HashMap<u32, Box<dyn Peer + Send>>,
  pub tx: Sender<HubEvent>,
}

impl EventHub {
  pub fn new(buffer: usize) -> (Self, Receiver<HubEvent>) {
    let (tx, rx) = mpsc::channel(buffer);
    (Self::with_tx(tx), rx)
  }

  pub fn with_tx(tx: Sender<HubEvent>) -> Self {
    EventHub {
      peers: HashMap::new(),
      tx,
    }
  }

  pub fn add_peer(&mut self, peer: Box<dyn Peer + Send>) -> Result<()> {
    match self.peers.entry(peer.id()) {
      Entry::Occupied(_) => Err(Box::new(Error::PeerAlreadyExist(peer.id()))),
      Entry::Vacant(e) => {
        e.insert(peer);
        Ok(())
      }
    }
  }

  pub fn remove_peer(&mut self, id: u32) -> Result<()> {
    match self.peers.remove(&id) {
      Some(mut p) => {
        p.stop();
        Ok(())
      }
      None => Err(Box::new(Error::PeerNotExist(id))),
    }
  }

  pub async fn write_to(&mut self, id: u32, data: Bytes) -> Result<()> {
    match self.peers.get_mut(&id) {
      Some(peer) => Ok(peer.write(data).await?),
      None => Err(Box::new(Error::PeerNotExist(id))),
    }
  }

  pub async fn echo(&mut self, msg: PeerMsg) -> Result<()> {
    self.write_to(msg.peer_id, msg.data).await
  }

  pub async fn broadcast<F>(&mut self, data: Bytes, selector: F) -> MultiResult<bool>
  where
    F: Fn(&Box<dyn Peer + Send>) -> bool,
  {
    let mut futures = HashMap::with_capacity(self.peers.len());
    let mut result = HashMap::with_capacity(self.peers.len());

    // broadcast
    for (id, p) in self.peers.iter_mut() {
      if selector(p) {
        futures.insert(*id, p.write(data.clone()));
      } else {
        result.insert(*id, Ok(false));
      };
    }

    // gather result
    for (id, f) in futures {
      let t = match f.await {
        Ok(_) => Ok(true),
        Err(e) => Err(e),
      };
      result.insert(id, t);
    }
    result
  }

  pub async fn broadcast_all(&mut self, data: Bytes) -> MultiResult<bool> {
    self.broadcast(data, |_| true).await
  }
}
