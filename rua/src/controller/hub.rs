use std::{
  cell::RefCell,
  collections::{hash_map::Entry, HashMap},
  sync::mpsc::{self, Receiver, Sender},
};

use crate::model::{Data, MultiResult, Peer, PeerMsg, Result, ServerError, ServerEvent};

pub struct MsgHub {
  name: String,
  peers: RefCell<HashMap<i32, Box<dyn Peer>>>,
  tx: Sender<ServerEvent>,
  rx: Receiver<ServerEvent>,
}

impl MsgHub {
  pub fn new() -> Self {
    let (tx, rx) = mpsc::channel();
    MsgHub {
      name: String::from("EventDrivenServer"),
      peers: RefCell::new(HashMap::new()),
      tx,
      rx,
    }
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn set_name(&mut self, name: &str) {
    self.name = name.to_string()
  }

  pub fn with_name(mut self, name: &str) -> Self {
    self.set_name(name);
    self
  }

  pub fn tx(&self) -> Sender<ServerEvent> {
    self.tx.clone()
  }

  pub fn recv(&self) -> ServerEvent {
    self.rx.recv().unwrap()
  }

  pub fn add_peer(&self, peer: Box<dyn Peer>) -> Result<()> {
    match self.peers.borrow_mut().entry(peer.id()) {
      Entry::Occupied(_) => {
        Err(Box::new(ServerError::PeerAlreadyExist(peer.id())))
        // the new peer will drop itself since it's not moved into the HashMap
      }
      Entry::Vacant(e) => e.insert(peer).start(),
    }
  }

  pub fn remove_peer(&self, id: i32) -> Result<()> {
    match self.peers.borrow_mut().remove(&id) {
      Some(_) => {
        // the target peer will drop itself since it's moved out of the HashMap
        Ok(())
      }
      None => Err(Box::new(ServerError::PeerNotExist(id))),
    }
  }

  pub fn stop(&self) {
    self.tx.send(ServerEvent::Stop).unwrap();
  }

  pub fn for_each_peer<F, T>(&self, f: F) -> MultiResult<T>
  where
    F: Fn(&mut Box<dyn Peer>) -> Result<T>,
  {
    let mut result = HashMap::with_capacity(self.peers.borrow().len());
    for (id, peer) in self.peers.borrow_mut().iter_mut() {
      result.insert(*id, f(peer));
    }
    result
  }

  pub fn apply_to<F, T>(&self, id: i32, f: F) -> Result<T>
  where
    F: FnOnce(&mut Box<dyn Peer>) -> Result<T>,
  {
    match self.peers.borrow_mut().get_mut(&id) {
      Some(peer) => f(peer),
      None => Err(Box::new(ServerError::PeerNotExist(id))),
    }
  }

  pub fn write_to(&self, id: i32, data: Data) -> Result<()> {
    self.apply_to(id, |p| p.write(data))
  }

  pub fn echo(&self, msg: PeerMsg) -> Result<()> {
    self.write_to(msg.peer_id, msg.data)
  }

  pub fn broadcast<F>(&self, data: Data, selector: F) -> MultiResult<bool>
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

  pub fn broadcast_all(&self, data: Data) -> MultiResult<bool> {
    self.broadcast(data, |_| true)
  }
}
