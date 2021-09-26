use std::{
    collections::HashMap,
    error::Error,
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
};

use crate::model::{GameServer, Peer, PeerMsg, ServerError, ServerEvent};

pub struct Server {
    name: String,
    peers: Mutex<HashMap<i32, Box<dyn Peer>>>,
    on_peer_msg_handler: Box<dyn Fn(PeerMsg)>,
    event_sender: Sender<ServerEvent>,
    event_receiver: Receiver<ServerEvent>,
}

impl Server {
    pub fn new() -> Server {
        let (event_sender, event_receiver) = mpsc::channel();
        Server {
            name: String::from("EventDrivenServer"),
            peers: Mutex::new(HashMap::new()),
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

        // close all peers
        for (_, peer) in self.peers.lock().unwrap().iter_mut() {
            peer.close().unwrap();
        }
    }

    pub fn on_peer_msg(&mut self, f: Box<dyn Fn(PeerMsg)>) -> &Self {
        self.on_peer_msg_handler = f;
        self
    }
}

impl GameServer for Server {
    fn add_peer(&self, mut peer: Box<dyn Peer>) -> i32 {
        let mut peers = self.peers.lock().unwrap();
        let new_peer_id = match peers.keys().max() {
            Some(max) => max + 1,
            None => 0,
        };
        peer.activate(new_peer_id, self.event_sender.clone());
        peer.start();
        peers.insert(new_peer_id, peer);
        new_peer_id
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

    fn for_each_peer(&self, mut f: Box<dyn FnMut(&Box<dyn Peer>)>) {
        for (_, peer) in self.peers.lock().unwrap().iter_mut() {
            f(peer)
        }
    }

    fn apply_to(
        &self,
        id: i32,
        mut f: Box<dyn FnMut(&Box<dyn Peer>)>,
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
