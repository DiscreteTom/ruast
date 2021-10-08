use std::error::Error;

use rua::{
  controller::EventHub,
  model::Event,
  peer::{FilePeer, StdioPeer},
};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
  let mut h = EventHub::new(256);

  h.add_peer(StdioPeer::new(0, h.tx(), 256))?;
  h.add_peer(FilePeer::new(1, "log.txt", 256)?)?;

  loop {
    match h.recv().await {
      Event::PeerMsg(msg) => {
        h.broadcast_all(msg.data).await;
      }
      _ => break,
    }
  }

  Ok(())
}
