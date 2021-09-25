use std::{
    collections::HashMap,
    error::Error,
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
};

use crate::model::{GameServer, Peer, PeerMsg, ServerError, ServerEvent};

pub struct Server<'a> {
    name: String,
    peers: Mutex<HashMap<i32, Box<dyn Peer<'a>>>>,
    on_peer_msg_handler: Box<dyn Fn(PeerMsg)>,
    event_sender: Sender<ServerEvent<'a>>,
    event_receiver: Receiver<ServerEvent<'a>>,
}

impl<'a> Server<'a> {
    pub fn new() -> Server<'a> {
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
        // process peer message and wait for stop
        loop {
            match self.event_receiver.recv().unwrap() {
                ServerEvent::Msg(msg) => (self.on_peer_msg_handler)(msg),
                ServerEvent::Stop => break,
            }
        }

        // close all peers
        self.peers.lock().unwrap().iter().map(|(_, peer)| {
            peer.close();
        });
    }

    pub fn on_peer_msg(&mut self, f: Box<dyn Fn(PeerMsg)>) -> &Self {
        self.on_peer_msg_handler = f;
        self
    }
}

impl<'a> GameServer<'a> for Server<'a> {
    fn add_peer(&self, peer: Box<dyn Peer<'a>>) -> i32 {
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
        self.event_sender.send(ServerEvent::Stop);
    }

    fn for_each_peer(&self, f: Box<dyn FnMut((&i32, &Box<dyn Peer>))>) {
        self.peers.lock().unwrap().iter().map(f);
    }

    fn apply_to(&self, id: i32, mut f: Box<dyn FnMut(&Box<dyn Peer>)>) -> Result<(), Box<dyn Error>> {
        match self.peers.lock().unwrap().get(&id) {
            Some(peer) => {
                f(peer);
                Ok(())
            }
            None => Err(Box::new(ServerError::PeerNotExist(id))),
        }
    }
}
