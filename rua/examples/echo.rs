use rua::{
  controller::EventHub,
  model::{HubEvent, Result},
  peer::StdioPeerBuilder,
};

#[tokio::main]
pub async fn main() -> Result<()> {
  let mut h = EventHub::new(256);

  h.add_peer(StdioPeerBuilder::new(0, h.tx_clone(), 256).build())?;

  loop {
    match h.recv().await {
      HubEvent::PeerMsg(msg) => h.echo(msg).await?,
      _ => break,
    }
  }

  Ok(())
}
