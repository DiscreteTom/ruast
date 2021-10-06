use std::error::Error;

use rua::{
  model::ServerEvent,
  peer::{FilePeer, StdioPeer},
  server::MsgHub,
};

fn main() -> Result<(), Box<dyn Error>> {
  let s = MsgHub::new();

  s.add_peer(StdioPeer::new(0, s.tx()))?;
  s.add_peer(FilePeer::new(1, "log.txt")?)?;

  loop {
    match s.recv() {
      ServerEvent::PeerMsg(msg) => {
        s.broadcast_all(msg.data);
      }
      _ => break,
    }
  }

  Ok(())
}
