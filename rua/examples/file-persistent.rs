use bytes::{BufMut, BytesMut};
use rua::{
  controller::PeerManager,
  model::{PeerBuilder, Result, ServerEvent},
  peer::{FilePeerBuilder, StdioPeerBuilder},
};
use tokio::sync::mpsc;

#[tokio::main]
pub async fn main() -> Result<()> {
  let mut pm = PeerManager::new();
  let (tx, mut rx) = mpsc::channel(256);

  pm.add_peer(
    StdioPeerBuilder::new()
      .id(0)
      .server_tx(tx.clone())
      .build()
      .await?,
  )?;
  pm.add_peer(
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
        pm.broadcast_all(msg.data).await;
      }
      ServerEvent::RemovePeer(id) => pm.remove_peer(id)?,
      _ => break,
    }
  }

  Ok(())
}
