use rua::{
  controller::EventHub,
  model::{HubEvent, Result},
  peer::{FilePeerBuilder, StdioPeerBuilder},
};

#[tokio::main]
pub async fn main() -> Result<()> {
  let mut h = EventHub::new(256);

  h.add_peer(StdioPeerBuilder::new(0, h.tx_clone(), 256).build())?;
  h.add_peer(
    FilePeerBuilder::new(1, "log.txt".to_string(), 256)
      .build()
      .await?,
  )?;

  loop {
    match h.recv().await {
      HubEvent::PeerMsg(msg) => {
        h.broadcast_all(msg.data).await;
      }
      HubEvent::RemovePeer(id) => h.remove_peer(id).await?,
      _ => break,
    }
  }

  Ok(())
}
