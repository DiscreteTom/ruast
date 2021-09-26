use std::{
    error::Error,
    fmt,
    sync::Arc,
    sync::{mpsc::Sender, Weak},
    time::SystemTime,
};

pub trait Peer<'a> {
    fn write(&mut self, data: Arc<&[u8]>) -> Result<(), Box<dyn Error>>;
    fn close(&mut self) -> Result<(), Box<dyn Error>>;
    fn start(&mut self);
    fn activate(&mut self, id: i32, msg_sender: Sender<ServerEvent<'a>>);
    fn id(&self) -> i32;
    fn set_tag(&mut self, tag: &str);
    fn tag(&self) -> &str;
}

pub struct PeerMsg<'a> {
    pub peer: Weak<dyn Peer<'a>>,
    pub data: Arc<&'a [u8]>,
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
        // match *self {
        //     ServerError::PeerNotExist(id) => write!(f, "peer not exist, id={}", id),
        // }
        write!(f, "123")
    }
}

impl Error for ServerError {}

pub trait GameServer<'a> {
    fn add_peer(&self, peer: Box<dyn Peer<'a>>) -> i32;
    fn remove_peer(&self, id: i32) -> Result<(), Box<dyn Error>>;
    fn stop(&self);
    fn for_each_peer(&self, f: Box<dyn FnMut(&Box<dyn Peer>)>);
    fn apply_to(&self, id: i32, f: Box<dyn FnMut(&Box<dyn Peer>)>) -> Result<(), Box<dyn Error>>;
}
