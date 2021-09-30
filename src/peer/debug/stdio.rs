use std::{
  error::Error,
  io,
  sync::{mpsc::Sender, Arc, Weak},
  thread,
  time::SystemTime,
};

use crate::model::{Peer, PeerMsg, ServerEvent};

pub struct StdioPeer {
  tag: String,
  id: i32,
}

impl Peer for StdioPeer {
  fn write(&mut self, data: Arc<[u8]>) -> Result<(), Box<dyn Error>> {
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
}

impl StdioPeer {
  pub fn new(id: i32, msg_sender: Sender<ServerEvent>) -> Result<Arc<dyn Peer>, Box<dyn Error>> {
    let p = Arc::new(StdioPeer {
      tag: String::from("stdio"),
      id,
    });
    let weak_p = Arc::downgrade(&p) as Weak<dyn Peer>;
    thread::spawn(move || {
      loop {
        // read line
        let mut line = String::new();
        io::stdin()
          .read_line(&mut line)
          .expect("Failed to read line");

        // send
        msg_sender
          .send(ServerEvent::Msg(PeerMsg {
            peer: weak_p.clone(),
            data: Arc::new(line.into_bytes()),
            time: SystemTime::now(),
          }))
          .unwrap()
      }
    });
    Ok(p)
  }
}
