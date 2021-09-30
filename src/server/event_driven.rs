use std::{
  collections::HashMap,
  error::Error,
  sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
  },
};

use crate::model::{GameServer, PeerMsg, PeerReader, PeerWriter, ServerError, ServerEvent};

pub struct Server {
  name: String,
  peer_writers: Mutex<HashMap<i32, Arc<dyn PeerWriter>>>,
  on_peer_msg_handler: Box<dyn Fn(PeerMsg)>,
  event_sender: Sender<ServerEvent>,
  event_receiver: Receiver<ServerEvent>,
}

impl Server {
  pub fn new() -> Server {
    let (event_sender, event_receiver) = mpsc::channel();
    Server {
      name: String::from("EventDrivenServer"),
      peer_writers: Mutex::new(HashMap::new()),
      on_peer_msg_handler: Box::new(|_| {}),
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

  pub fn on_peer_msg(&mut self, f: Box<dyn Fn(PeerMsg)>) -> &Self {
    self.on_peer_msg_handler = f;
    self
  }
}

impl GameServer for Server {
  fn new_peer(
    &self,
    generator: Box<dyn Fn(i32, Sender<ServerEvent>) -> (Arc<dyn PeerWriter>, Box<dyn PeerReader>)>,
  ) -> i32 {
    // get new peer id, starts from 0
    let mut peers = self.peer_writers.lock().unwrap();
    let new_peer_id = match peers.keys().max() {
      Some(max) => max + 1,
      None => 0,
    };
    let (p_writer, mut p_reader) = generator(new_peer_id, self.event_sender.clone());

    // thread pool here
    p_reader.start();

    peers.insert(new_peer_id, p_writer);
    new_peer_id
  }

  fn remove_peer(&self, id: i32) -> Result<(), Box<dyn Error>> {
    let mut peers = self.peer_writers.lock().unwrap();
    match peers.remove(&id) {
      Some(_) => Ok(()),
      None => Err(Box::new(ServerError::PeerNotExist(id))),
    }
  }

  fn stop(&self) {
    self.event_sender.send(ServerEvent::Stop).unwrap();
  }

  fn for_each_peer(&self, mut f: Box<dyn FnMut(&Arc<dyn PeerWriter>)>) {
    for (_, peer) in self.peer_writers.lock().unwrap().iter_mut() {
      f(peer)
    }
  }

  fn apply_to(
    &self,
    id: i32,
    mut f: Box<dyn FnMut(&Arc<dyn PeerWriter>)>,
  ) -> Result<(), Box<dyn Error>> {
    match self.peer_writers.lock().unwrap().get(&id) {
      Some(peer) => {
        f(peer);
        Ok(())
      }
      None => Err(Box::new(ServerError::PeerNotExist(id))),
    }
  }
}
