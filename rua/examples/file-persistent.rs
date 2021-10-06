use std::error::Error;

use rua::{
  controller::EventHub,
  model::Event,
  peer::{FilePeer, StdioPeer},
};

fn main() -> Result<(), Box<dyn Error>> {
  let h = EventHub::new();

  h.add_peer(StdioPeer::new(0, h.tx()))?;
  h.add_peer(FilePeer::new(1, "log.txt")?)?;

  loop {
    match h.recv() {
      Event::PeerMsg(msg) => {
        h.broadcast_all(msg.data);
      }
      _ => break,
    }
  }

  Ok(())
}
