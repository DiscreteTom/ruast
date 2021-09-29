use std::{error::Error, io, sync::{Arc, Weak, mpsc::Sender}, time::SystemTime};

use crate::model::{PeerMsg, PeerReader, PeerWriter, ServerEvent};

pub struct StdioPeerWriter {
  tag: String,
  id: i32,
}
pub struct StdioPeerReader {
  msg_sender: Sender<ServerEvent>,
  writer: Arc<StdioPeerWriter>,
}

impl Drop for StdioPeerWriter {
  fn drop(&mut self) {}
}

impl PeerWriter for StdioPeerWriter {
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

pub fn new(
  id: i32,
  msg_sender: Sender<ServerEvent>,
) -> (Arc<StdioPeerWriter>, Box<StdioPeerReader>) {
  let reader = StdioPeerReader {
    msg_sender,
    writer: Arc::new(StdioPeerWriter {
      tag: String::from("stdio"),
      id,
    }),
  };
  (reader.writer.clone(), Box::new(reader))
}


impl PeerReader for StdioPeerReader {
  fn start(&mut self) {
    loop {
      // read line
      let mut line = String::new();
      io::stdin()
        .read_line(&mut line)
        .expect("Failed to read line");

      // send
      self
        .msg_sender
        .send(ServerEvent::Msg(PeerMsg {
          peer: Arc::downgrade(&self.writer) as Weak<dyn PeerWriter>,
          data: Arc::new(line.into_bytes()),
          time: SystemTime::now(),
        }))
        .unwrap()
    }
  }
}
