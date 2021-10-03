use std::{
  collections::{hash_map::Entry, HashMap},
  error::Error,
  sync::{
    mpsc::{self, Receiver, Sender},
    Mutex,
  },
};

use crate::model::{GameServer, Peer, PeerMsg, ServerError, ServerEvent};

pub struct EventDrivenServer {
  name: String,
  peers: Mutex<HashMap<i32, Box<dyn Peer>>>,
  on_peer_msg_handler: fn(PeerMsg),
  tx: Sender<ServerEvent>,
  rx: Receiver<ServerEvent>,
}

impl EventDrivenServer {
  pub fn new() -> Self {
    let (tx, rx) = mpsc::channel();
    EventDrivenServer {
      name: String::from("EventDrivenServer"),
      peers: Mutex::new(HashMap::new()),
      on_peer_msg_handler: |_| {},
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
        ServerEvent::PeerMsg(msg) => (self.on_peer_msg_handler)(msg),
        ServerEvent::Stop => break,
      }
    }
  }

  pub fn on_peer_msg(&mut self, f: fn(PeerMsg)) -> &Self {
    self.on_peer_msg_handler = f;
    self
  }
}

impl GameServer for EventDrivenServer {
  fn add_peer(&self, id: i32, peer: Box<dyn Peer>) -> Result<(), Box<dyn Error>> {
    match self.peers.lock().unwrap().entry(id) {
      Entry::Occupied(_) => {
        Err(Box::new(ServerError::PeerAlreadyExist(id)))
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

  fn for_each_peer(&self, f: fn(&Box<dyn Peer>)) {
    for (_, peer) in self.peers.lock().unwrap().iter() {
      // peer.send(PeerEvent::Apply(f)).unwrap();
      f(peer)
    }
  }

  fn apply_to(&self, id: i32, f: fn(&Box<dyn Peer>)) -> Result<(), Box<dyn Error>> {
    match self.peers.lock().unwrap().get(&id) {
      Some(peer) => {
        f(peer);
        Ok(())
      }
      None => Err(Box::new(ServerError::PeerNotExist(id))),
    }
  }
}
