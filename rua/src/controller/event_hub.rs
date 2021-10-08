use bytes::Bytes;
use std::{
  cell::RefCell,
  collections::{hash_map::Entry, HashMap},
};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::model::{Error, HubEvent, MultiResult, Peer, PeerMsg, Result};

pub struct EventHub {
  peers: RefCell<HashMap<i32, Box<dyn Peer>>>,
  tx: Sender<HubEvent>,
  rx: Receiver<HubEvent>,
}

impl EventHub {
  pub fn new(buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(buffer);
    EventHub {
      peers: RefCell::new(HashMap::new()),
      tx,
      rx,
    }
  }

  pub fn tx_clone(&self) -> Sender<HubEvent> {
    self.tx.clone()
  }

  pub async fn recv(&mut self) -> HubEvent {
    self.rx.recv().await.unwrap()
  }

  pub fn add_peer(&self, peer: Box<dyn Peer>) -> Result<()> {
    match self.peers.borrow_mut().entry(peer.id()) {
      Entry::Occupied(_) => Err(Box::new(Error::PeerAlreadyExist(peer.id()))),
      Entry::Vacant(e) => e.insert(peer).start(),
    }
  }

  pub fn remove_peer(&self, id: i32) -> Result<()> {
    match self.peers.borrow_mut().remove(&id) {
      Some(_) => Ok(()),
      None => Err(Box::new(Error::PeerNotExist(id))),
    }
  }

  pub async fn stop(&self) {
    self.tx.send(HubEvent::Stop).await.unwrap();
  }

  pub async fn write_to(&self, id: i32, data: Bytes) -> Result<()> {
    match self.peers.borrow_mut().get_mut(&id) {
      Some(peer) => Ok(peer.tx().send(data).await?),
      None => Err(Box::new(Error::PeerNotExist(id))),
    }
  }

  pub async fn echo(&self, msg: PeerMsg) -> Result<()> {
    self.write_to(msg.peer_id, msg.data).await
  }

  pub async fn broadcast<F>(&self, data: Bytes, selector: F) -> MultiResult<bool>
  where
    F: Fn(&Box<dyn Peer>) -> bool,
  {
    let mut result = HashMap::with_capacity(self.peers.borrow().len());
    for (id, p) in self.peers.borrow_mut().iter_mut() {
      let t = if selector(p) {
        match p.tx().send(data.clone()).await {
          Ok(_) => Ok(true),
          Err(e) => Err(Box::new(e) as Box<dyn std::error::Error>),
        }
      } else {
        Ok(false)
      };
      result.insert(*id, t);
    }
    result
  }

  pub async fn broadcast_all(&self, data: Bytes) -> MultiResult<bool> {
    self.broadcast(data, |_| true).await
  }
}
