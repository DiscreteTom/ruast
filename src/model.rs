use std::{
    error::Error,
    fmt,
    sync::Arc,
    sync::{mpsc::Sender, Weak},
    time::SystemTime,
};

pub trait Peer<'a> {
    fn write(&self, data: &Arc<&[u8]>) -> Result<(), Box<dyn Error>>;
    fn close(&self) -> Result<(), Box<dyn Error>>;
    fn start(&self);
    fn activate(&self, id: i32, msg_sender: Sender<ServerEvent>);
    fn id(&self) -> i32;
    fn set_tag(&self, tag: &str);
    fn tag(&'a self) -> &'a str;
}

pub struct PeerMsg<'a> {
    pub peer: Weak<dyn Peer<'a>>,
    pub data: Arc<[u8]>,
    pub time: SystemTime,
}

pub enum ServerEvent<'a> {
    Msg(PeerMsg<'a>),
    Stop,
}

#[derive(Debug)]
pub enum ServerError {
    PeerNotExist(i32),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ServerError::PeerNotExist(id) => write!(f, "peer not exist, id={}", id),
        }
    }
}

pub trait GameServer<'a> {
    fn add_peer(&self, peer: Box<dyn Peer<'a>>) -> i32;
    fn remove_peer(&self, id: i32) -> Result<(), Box<dyn Error>>;
    fn stop(&self);
    fn for_each_peer(&self, f: Box<dyn Fn(dyn Peer)>);
    fn peer(&self, id: i32) -> Option<Weak<dyn Peer>>;
}
