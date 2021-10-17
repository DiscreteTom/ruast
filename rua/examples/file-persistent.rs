use bytes::{BufMut, BytesMut};
use rua::{
  controller::EventHub,
  model::{PeerBuilder, Result, ServerEvent},
  peer::{FilePeerBuilder, StdioPeerBuilder},
};
use tokio::sync::mpsc;

#[tokio::main]
pub async fn main() -> Result<()> {
  let mut h = EventHub::new();
  let (tx, mut rx) = mpsc::channel(256);

  h.add_peer(
    StdioPeerBuilder::new()
      .id(0)
      .server_tx(tx.clone())
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
    match rx.recv().await.unwrap() {
      ServerEvent::PeerMsg(msg) => {
        h.broadcast_all(msg.data).await;
      }
      ServerEvent::RemovePeer(id) => h.remove_peer(id)?,
      _ => break,
    }
  }

  Ok(())
}
