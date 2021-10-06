use std::{
  io,
  sync::{mpsc::Sender, Arc},
  thread,
  time::SystemTime,
};

use crate::model::{Data, EventType, Peer, PeerMsg, Result};

pub struct StdioPeer {
  tag: String,
  id: i32,
  server_tx: Sender<EventType>,
}

impl StdioPeer {
  pub fn new(id: i32, server_tx: Sender<EventType>) -> Box<dyn Peer> {
    Box::new(StdioPeer {
      tag: String::from("stdio"),
      id,
      server_tx,
    })
  }
}

impl Peer for StdioPeer {
  fn write(&mut self, data: Data) -> Result<()> {
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
  fn start(&mut self) -> Result<()> {
    let stx = self.server_tx.clone();
    let id = self.id;
    thread::spawn(move || {
      loop {
        // read line
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
          break;
        } else {
          // send
          stx
            .send(EventType::PeerMsg(PeerMsg {
              peer_id: id,
              data: Arc::new(line.into_bytes()),
              time: SystemTime::now(),
            }))
            .unwrap()
        }
      }
    });
    Ok(())
  }
}
