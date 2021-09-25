use std::{
    collections::HashMap,
    error::Error,
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex, Weak,
    },
};

use crate::model::{GameServer, Peer, PeerMsg};

pub enum ServerEvent<'a> {
    Msg(PeerMsg<'a>),
    Stop,
}

pub struct Server<'a> {
    name: String,
    peers: Mutex<HashMap<i32, Box<dyn Peer<'a>>>>,
    on_peer_msg_handler: dyn Fn(PeerMsg),
    event_sender: Sender<ServerEvent<'a>>,
    event_receiver: Receiver<ServerEvent<'a>>,
}

impl<'a> Server<'a> {
    pub fn new() -> Server<'a> {
        let (event_sender, event_receiver) = mpsc::channel();
        Server {
            name: String::from("EventDrivenServer"),
            peers: Mutex::new(HashMap::new()),
            on_peer_msg_handler: || {},
            event_sender,
            event_receiver,
        }
    }

    pub fn start(&self) {
        // process peer message and wait for stop
        loop {
            match self.event_receiver.recv().unwrap() {
                ServerEvent::Msg(msg) => self.on_peer_msg_handler.call(msg),
                ServerEvent::Stop => break,
            }
        }

        // close all peers
        self.peers.lock().unwrap().iter().map(|(_, peer)| {
            peer.close();
        })
    }

    pub fn on_peer_msg(&self, f: dyn Fn(PeerMsg)) -> Self {
        self.on_peer_msg_handler = f;
        self
    }
}

impl<'a> GameServer for Server<'a> {
    fn add_peer(&self, peer: Box<dyn Peer>) -> i32 {
        let peers = self.peers.lock().unwrap();
        let new_peer_id = match peers.keys().max() {
            Some(max) => max + 1,
            None => 0,
        };
        peers.insert(new_peer_id, peer);
        peer.activate(new_peer_id, self.event_sender.clone());
        peer.start();
        new_peer_id
    }

    fn remove_peer(&self, id: i32) -> Result<(), Box<dyn Error>> {
        let peers = self.peers.lock().unwrap();
        match peers.remove(&id) {
            Some(_) => Ok(()),
            None => Err("peer not exist"),
        }
    }

    fn stop(&self) {
        self.event_sender.send(ServerEvent::Stop);
    }

    fn for_each_peer(&self, f: dyn Fn(dyn Peer)) {
        self.peers.lock().unwrap().iter().map(f)
    }

    fn peer(&self, id: i32) -> Option<Weak<dyn Peer>> {
        self.peers.lock().unwrap().get(&id)
    }
}
