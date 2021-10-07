use std::{
  io::{self, Write},
  sync::{mpsc::Sender, Arc},
  thread,
};

use crate::model::{Data, Event, Peer, PeerMsg, Result};

pub struct StdioPeer {
  tag: String,
  id: i32,
  hub_tx: Sender<Event>,
}

impl StdioPeer {
  pub fn new(id: i32, hub_tx: Sender<Event>) -> Box<dyn Peer> {
    Box::new(StdioPeer {
      tag: String::from("stdio"),
      id,
      hub_tx,
    })
  }
}

impl Peer for StdioPeer {
  fn write(&mut self, data: Data) -> Result<()> {
    print!("{}", String::from_utf8_lossy(&data));
    io::stdout().flush().unwrap();
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
    let hub_tx = self.hub_tx.clone();
    let id = self.id;
    thread::spawn(move || {
      loop {
        // read line
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
          break;
        } else {
          // send
          hub_tx
            .send(Event::PeerMsg(PeerMsg {
              peer_id: id,
              data: Arc::new(line.into_bytes()),
            }))
            .unwrap()
        }
      }
    });
    Ok(())
  }
}
