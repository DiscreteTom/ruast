use std::{error::Error, sync::Arc, sync::Weak, time::SystemTime};

pub trait Peer<'a> {
    fn write(&self, data: &Arc<&[u8]>) -> Result<(), Box<dyn Error>>;
    fn close(&self) -> Result<(), Box<dyn Error>>;
    fn start(&self);
    fn activate(&self, id: i32);
    fn id(&self) -> i32;
    fn set_tag(&self, tag: &str);
    fn tag(&self) -> &'a str;
}

pub struct PeerMsg<'a> {
    pub peer: Weak<dyn Peer<'a>>,
    pub data: Arc<[u8]>,
    pub time: SystemTime,
}

pub trait GameServer {
    fn add_peer(&self, peer: Box<dyn Peer>) -> i32;
    fn remove_peer(&self, id: i32) -> Result<(), Box<dyn Error>>;
    fn stop(&self);
    fn for_each_peer(&self, f: dyn Fn(dyn Peer));
    fn peer(&self, id: i32) -> Option<Weak<dyn Peer>>;
}
