use std::{error::Error, sync::Arc, sync::Weak, time::SystemTime};

pub trait Peer<'a> {
    fn write(&self, data: &Arc<&[u8]>) -> Result<(), Box<dyn Error>>;
    fn close(&self) -> Result<(), Box<dyn Error>>;
    fn start(&self);
    fn set_id(&self, id: i32);
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
    fn add_peer(&self, peer: dyn Peer) -> i32;
    fn remove_peer(&self, id: i32) -> Result<(), Box<dyn Error>>;
    fn append_peer_msg(&self, peer: dyn Peer, data: [u8]);
    fn stop(&self);
    fn for_each_peer(&self, f: dyn Fn(dyn Peer));
    fn peer(&self, id: i32) -> Option<Weak<dyn Peer>>;
}
