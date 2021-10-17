use rua::{
  controller::EventHub,
  model::{HubEvent, PeerBuilder, Result},
  peer::StdioPeerBuilder,
};

#[tokio::main]
pub async fn main() -> Result<()> {
  let (mut h, mut rx) = EventHub::new(256);

  h.add_peer(
    StdioPeerBuilder::new()
      .output_selector(|data| !data.starts_with(b"#"))
      .id(0)
      .hub_tx(h.tx.clone())
      .build()
      .await?,
  )?;

  loop {
    match rx.recv().await.unwrap() {
      HubEvent::PeerMsg(msg) => h.echo(msg).await?,
      HubEvent::RemovePeer(id) => h.remove_peer(id)?,
      _ => break,
    }
  }

  Ok(())
}
