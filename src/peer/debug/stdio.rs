use std::{
  error::Error,
  io,
  sync::{mpsc::Sender, Arc},
  thread,
  time::SystemTime,
};

use crate::model::{Peer, PeerMsg, ServerEvent};

pub struct StdioPeer {
  tag: String,
  id: i32,
  server_tx: Sender<ServerEvent>,
}

impl StdioPeer {
  pub fn new(id: i32, server_tx: Sender<ServerEvent>) -> Box<dyn Peer> {
    Box::new(StdioPeer {
      tag: String::from("stdio"),
      id,
      server_tx,
    })
  }
}

impl Peer for StdioPeer {
  fn write(&mut self, data: Arc<Vec<u8>>) -> Result<(), Box<dyn Error>> {
    println!("{}", String::from_utf8_lossy(&data));
    Ok(())
  }
  fn id(&self) -> i32 {
    self.id
  }
  fn set_tag(&mut self, tag: &str) {
    self.tag = String::from(tag);
  }
  fn tag(&self) -> &str {
    &self.tag
  }
  fn start(&mut self) -> Result<(), Box<dyn Error>> {
    let stx = self.server_tx.clone();
    let id = self.id;
    thread::spawn(move || {
      loop {
        // read line
        let mut line = String::new();
        io::stdin()
          .read_line(&mut line)
          .expect("Failed to read line");

        // send
        stx
          .send(ServerEvent::PeerMsg(PeerMsg {
            peer_id: id,
            data: Arc::new(line.into_bytes()),
            time: SystemTime::now(),
          }))
          .unwrap()
      }
    });
    Ok(())
  }
}