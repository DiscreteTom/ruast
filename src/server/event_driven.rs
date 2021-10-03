use std::{
  collections::{hash_map::Entry, HashMap},
  error::Error,
  sync::{
    mpsc::{self, Receiver, Sender},
    Mutex,
  },
};

use crate::model::{GameServer, Peer, PeerMsg, ServerError, ServerEvent};

pub struct EventDrivenServer<'a> {
  name: String,
  peers: Mutex<HashMap<i32, Box<dyn Peer>>>,
  on_peer_msg_handler: &'a dyn Fn(PeerMsg, &Self),
  tx: Sender<ServerEvent>,
  rx: Receiver<ServerEvent>,
}

impl<'a> EventDrivenServer<'a> {
  pub fn new() -> Self {
    let (tx, rx) = mpsc::channel();
    EventDrivenServer {
      name: String::from("EventDrivenServer"),
      peers: Mutex::new(HashMap::new()),
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
    match self.peers.lock().unwrap().entry(peer.id()) {
      Entry::Occupied(_) => {
        Err(Box::new(ServerError::PeerAlreadyExist(peer.id())))
        // the new peer will drop itself since it's not moved into the HashMap
      }
      Entry::Vacant(e) => e.insert(peer).start(),
    }
  }

  fn remove_peer(&self, id: i32) -> Result<(), Box<dyn Error>> {
    match self.peers.lock().unwrap().remove(&id) {
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

  fn for_each_peer<F>(&self, f: F) -> Vec<(i32, Result<(), Box<dyn Error>>)>
  where
    F: Fn(&mut Box<dyn Peer>) -> Result<(), Box<dyn Error>>,
  {
    let mut peers = self.peers.lock().unwrap();
    let mut result = Vec::with_capacity(peers.len());
    for (id, peer) in peers.iter_mut() {
      // peer.send(PeerEvent::Apply(f)).unwrap();
      result.push((*id, f(peer)));
    }
    result
  }

  fn apply_to<F>(&self, id: i32, f: F) -> Result<(), Box<dyn Error>>
  where
    F: FnOnce(&mut Box<dyn Peer>) -> Result<(), Box<dyn Error>>,
  {
    match self.peers.lock().unwrap().get_mut(&id) {
      Some(peer) => f(peer),
      None => Err(Box::new(ServerError::PeerNotExist(id))),
    }
  }

  fn write_to(&self, id: i32, data: std::sync::Arc<Vec<u8>>) -> Result<(), Box<dyn Error>> {
    match self.peers.lock().unwrap().get_mut(&id) {
      None => Err(Box::new(ServerError::PeerNotExist(id))),
      Some(peer) => peer.write(data),
    }
  }
}
