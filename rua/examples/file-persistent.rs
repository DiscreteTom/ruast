use bytes::{BufMut, BytesMut};
use rua::{
  controller::EventHub,
  model::{HubEvent, PeerBuilder, Result},
  peer::{FilePeerBuilder, StdioPeerBuilder},
};

#[tokio::main]
pub async fn main() -> Result<()> {
  let mut h = EventHub::new(256);

  h.add_peer(
    StdioPeerBuilder::new()
      .id(0)
      .hub_tx(h.tx.clone())
      .build()
      .await?,
  )?;
  h.add_peer(
    FilePeerBuilder::new()
      .filename("log.txt".to_string())
      .separator("\n")
      .transformer(|data| {
        let prepend = b">> (";
        let append = b");";
        let mut buf = BytesMut::with_capacity(prepend.len() + data.len() + append.len());
        buf.put(&prepend[..]);
        buf.put(data);
        buf.put(&append[..]);
        buf.freeze()
      })
      .id(1)
      .build()
      .await?,
  )?;

  loop {
    match h.recv().await {
      HubEvent::PeerMsg(msg) => {
        h.broadcast_all(msg.data).await;
      }
      HubEvent::RemovePeer(id) => h.remove_peer(id)?,
      _ => break,
    }
  }

  Ok(())
}
