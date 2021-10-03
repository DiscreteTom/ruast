use std::{
  cell::RefCell,
  collections::{hash_map::Entry, HashMap},
  error::Error,
  sync::mpsc::{self, Receiver, Sender},
};

use crate::model::{GameServer, Peer, PeerMsg, ServerError, ServerEvent};

pub struct EventDrivenServer<'a> {
  name: String,
  peers: RefCell<HashMap<i32, Box<dyn Peer>>>,
  on_peer_msg_handler: &'a dyn Fn(PeerMsg, &Self),
  tx: Sender<ServerEvent>,
  rx: Receiver<ServerEvent>,
}

impl<'a> EventDrivenServer<'a> {
  pub fn new() -> Self {
    let (tx, rx) = mpsc::channel();
    EventDrivenServer {
      name: String::from("EventDrivenServer"),
      peers: RefCell::new(HashMap::new()),
      on_peer_msg_handler: &|_, _| {},
      tx,
      rx,
    }
  }

  pub fn tx(&self) -> Sender<ServerEvent> {
    self.tx.clone()
  }

  pub fn start(&self) {
    println!("{} is running...", self.name);

    // process peer message and wait for stop
    loop {
      match self.rx.recv().unwrap() {
        ServerEvent::PeerMsg(msg) => (self.on_peer_msg_handler)(msg, self),
        ServerEvent::Stop => break,
      }
    }
  }

  pub fn on_peer_msg(&mut self, f: &'a dyn Fn(PeerMsg, &Self)) -> &Self {
    self.on_peer_msg_handler = f;
    self
  }
}

impl<'a> GameServer for EventDrivenServer<'a> {
  fn add_peer(&self, peer: Box<dyn Peer>) -> Result<(), Box<dyn Error>> {
    match self.peers.borrow_mut().entry(peer.id()) {
      Entry::Occupied(_) => {
        Err(Box::new(ServerError::PeerAlreadyExist(peer.id())))
        // the new peer will drop itself since it's not moved into the HashMap
      }
      Entry::Vacant(e) => e.insert(peer).start(),
    }
  }

  fn remove_peer(&self, id: i32) -> Result<(), Box<dyn Error>> {
    match self.peers.borrow_mut().remove(&id) {
      Some(_) => {
        // the target peer will drop itself since it's moved out of the HashMap
        Ok(())
      }
      None => Err(Box::new(ServerError::PeerNotExist(id))),
    }
  }

  fn stop(&self) {
    self.tx.send(ServerEvent::Stop).unwrap();
  }

  fn for_each_peer<F, T>(&self, f: F) -> Vec<(i32, Result<T, Box<dyn Error>>)>
  where
    F: Fn(&mut Box<dyn Peer>) -> Result<T, Box<dyn Error>>,
  {
    let mut result = Vec::with_capacity(self.peers.borrow().len());
    for (id, peer) in self.peers.borrow_mut().iter_mut() {
      // peer.send(PeerEvent::Apply(f)).unwrap();
      result.push((*id, f(peer)));
    }
    result
  }

  fn apply_to<F, T>(&self, id: i32, f: F) -> Result<T, Box<dyn Error>>
  where
    F: FnOnce(&mut Box<dyn Peer>) -> Result<T, Box<dyn Error>>,
  {
    match self.peers.borrow_mut().get_mut(&id) {
      Some(peer) => f(peer),
      None => Err(Box::new(ServerError::PeerNotExist(id))),
    }
  }
}
