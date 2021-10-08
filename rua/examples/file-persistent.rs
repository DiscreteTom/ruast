use std::error::Error;

use rua::{
  controller::EventHub,
  model::HubEvent,
  peer::{FilePeer, StdioPeer},
};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
  let mut h = EventHub::new(256);

  h.add_peer(StdioPeer::new(0, h.tx_clone(), 256).build())?;
  h.add_peer(FilePeer::new(1, "log.txt", 256)?)?;

  loop {
    match h.recv().await {
      HubEvent::PeerMsg(msg) => {
        h.broadcast_all(msg.data).await;
      }
      _ => break,
    }
  }

  Ok(())
}
