use std::{
  error::Error,
  io,
  sync::{mpsc::Sender, Arc},
  time::SystemTime,
};

use crate::model::{Peer, PeerMsg, ServerEvent};

pub struct StdioPeer {
  msg_sender: Sender<ServerEvent>,
  tag: String,
  id: i32,
}

impl StdioPeer {
  pub fn new(id: i32, msg_sender: Sender<ServerEvent>) -> StdioPeer {
    StdioPeer {
      msg_sender,
      tag: String::from("stdio"),
      id,
    }
  }
}

impl Peer for StdioPeer {
  fn write(&mut self, data: Arc<[u8]>) -> Result<(), Box<dyn Error>> {
    println!("{}", String::from_utf8_lossy(&data));
    Ok(())
  }
  fn close(&mut self) -> Result<(), Box<dyn Error>> {
    Ok(())
  }
  fn start(&mut self) {
    loop {
      let mut line = String::new();
      io::stdin()
        .read_line(&mut line)
        .expect("Failed to read line");
      // self.msg_sender
      //   .send(ServerEvent::Msg(PeerMsg {
      //     peer: self,
      //     data: Arc::new(line.as_bytes()),
      //     time: SystemTime::now(),
      //   }))
      //   .unwrap(),
    }
  }
  fn id(&self) -> i32 {
    self.id
  }
  fn set_tag(&mut self, tag: &str) {
    self.tag = tag.to_string();
  }
  fn tag(&self) -> &str {
    &self.tag
  }
}
