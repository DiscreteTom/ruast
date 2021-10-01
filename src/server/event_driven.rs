use std::{
  collections::HashMap,
  error::Error,
  sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
  },
};

use crate::model::{GameServer, Peer, PeerMsg, ServerError, ServerEvent};

pub struct EventDrivenServer {
  name: String,
  peers: Mutex<HashMap<String, Arc<Mutex<dyn Peer>>>>,
  on_peer_msg_handler: fn(PeerMsg),
  event_sender: Sender<ServerEvent>,
  event_receiver: Receiver<ServerEvent>,
}

impl EventDrivenServer {
  pub fn new() -> (EventDrivenServer, Sender<ServerEvent>) {
    let (event_sender, event_receiver) = mpsc::channel();
    (
      EventDrivenServer {
        name: String::from("EventDrivenServer"),
        peers: Mutex::new(HashMap::new()),
        on_peer_msg_handler: |_| {},
        event_sender: event_sender.clone(),
        event_receiver,
      },
      event_sender,
    )
  }

  pub fn start(&self) {
    println!("{} is running...", self.name);

    // process peer message and wait for stop
    loop {
      match self.event_receiver.recv().unwrap() {
        ServerEvent::Msg(msg) => (self.on_peer_msg_handler)(msg),
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
  fn add_peer(&self, peer: Arc<Mutex<dyn Peer>>) -> Result<(), Box<dyn Error>> {
    let mut peers = self.peers.lock().unwrap();
    let id = peer.lock().unwrap().id().to_string();

    match peers.contains_key(&id) {
      true => Err(Box::new(ServerError::PeerAlreadyExist(id))),
      false => {
        peers.insert(id, peer);
        Ok(())
      }
    }
  }

  fn remove_peer(&self, id: &str) -> Result<(), Box<dyn Error>> {
    let mut peers = self.peers.lock().unwrap();
    let id = id.to_string();

    match peers.remove(&id) {
      Some(_) => Ok(()),
      None => Err(Box::new(ServerError::PeerNotExist(id))),
    }
  }

  fn stop(&self) {
    self.event_sender.send(ServerEvent::Stop).unwrap();
  }

  fn for_each_peer(&self, f: fn(Arc<Mutex<dyn Peer>>)) {
    for (_, peer) in self.peers.lock().unwrap().iter() {
      f(peer.clone())
    }
  }

  fn apply_to(&self, id: &str, f: fn(Arc<Mutex<dyn Peer>>)) -> Result<(), Box<dyn Error>> {
    let id = id.to_string();

    match self.peers.lock().unwrap().get(&id) {
      Some(peer) => {
        f(peer.clone());
        Ok(())
      }
      None => Err(Box::new(ServerError::PeerNotExist(id))),
    }
  }
}
