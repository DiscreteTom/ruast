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
  peers: Mutex<HashMap<i32, Arc<Mutex<dyn Peer>>>>,
  on_peer_msg_handler: fn(PeerMsg),
  event_sender: Sender<ServerEvent>,
  event_receiver: Receiver<ServerEvent>,
}

impl EventDrivenServer {
  pub fn new() -> EventDrivenServer {
    let (event_sender, event_receiver) = mpsc::channel();
    EventDrivenServer {
      name: String::from("EventDrivenServer"),
      peers: Mutex::new(HashMap::new()),
      on_peer_msg_handler: |_| {},
      event_sender,
      event_receiver,
    }
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
  fn new_peer<F>(&self, generator: F) -> Result<i32, Box<dyn Error>>
  where
    F: Fn(i32, Sender<ServerEvent>) -> Result<Arc<Mutex<dyn Peer>>, Box<dyn Error>>,
  {
    // get new peer id, starts from 0
    let mut peers = self.peers.lock().unwrap();
    let new_peer_id = match peers.keys().max() {
      Some(max) => max + 1,
      None => 0,
    };

    peers.insert(
      new_peer_id,
      generator(new_peer_id, self.event_sender.clone())?,
    );
    Ok(new_peer_id)
  }

  fn remove_peer(&self, id: i32) -> Result<(), Box<dyn Error>> {
    let mut peers = self.peers.lock().unwrap();
    match peers.remove(&id) {
      Some(_) => Ok(()),
      None => Err(Box::new(ServerError::PeerNotExist(id))),
    }
  }

  fn stop(&self) {
    self.event_sender.send(ServerEvent::Stop).unwrap();
  }

  fn for_each_peer(&self, mut f: Box<dyn FnMut(&Arc<Mutex<dyn Peer>>)>) {
    for (_, peer) in self.peers.lock().unwrap().iter_mut() {
      f(peer)
    }
  }

  fn apply_to(
    &self,
    id: i32,
    mut f: Box<dyn FnMut(&Arc<Mutex<dyn Peer>>)>,
  ) -> Result<(), Box<dyn Error>> {
    match self.peers.lock().unwrap().get(&id) {
      Some(peer) => {
        f(peer);
        Ok(())
      }
      None => Err(Box::new(ServerError::PeerNotExist(id))),
    }
  }
}
