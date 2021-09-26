use std::{
  error::Error,
  io,
  sync::{mpsc::Sender, Arc},
  time::SystemTime,
};

use crate::model::{Peer, PeerMsg, ServerEvent};

pub struct StdioPeer {
  msg_sender: Option<Sender<ServerEvent>>,
  tag: String,
  id: Option<i32>,
}

impl StdioPeer {
  pub fn new() -> StdioPeer {
    StdioPeer {
      msg_sender: None,
      tag: String::from("stdio"),
      id: None,
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
      match self.msg_sender {
        // Some(sender) => sender
        //   .send(ServerEvent::Msg(PeerMsg {
        //     peer: self,
        //     data: Arc::new(line.as_bytes()),
        //     time: SystemTime::now(),
        //   }))
        //   .unwrap(),
        Some(_)=>(),
        None => continue,
      }
    }
  }
  fn activate(&mut self, id: i32, msg_sender: Sender<ServerEvent>) {
    self.id = Some(id);
    self.msg_sender = Some(msg_sender);
  }
  fn id(&self) -> i32 {
    self.id.unwrap()
  }
  fn set_tag(&mut self, tag: &str) {
    self.tag = tag.to_string();
  }
  fn tag(&self) -> &str {
    &self.tag
  }
}
