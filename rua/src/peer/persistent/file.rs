use tokio::{
  io::AsyncWriteExt,
  sync::mpsc::{self, Receiver, Sender},
};

use crate::{
  impl_peer_builder,
  model::{Peer, PeerEvent, Result},
};

pub struct FilePeerBuilder {
  id: Option<u32>,
  sink: Option<Sender<PeerEvent>>,
  tag: String,

  rx: Receiver<PeerEvent>,
  tx: Sender<PeerEvent>,
  filename: Option<String>,
}

impl FilePeerBuilder {
  impl_peer_builder!(all);

  pub fn new(buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(buffer);

    Self {
      tx,
      rx,
      id: None,
      sink: None,
      tag: String::from("file"),
      filename: None,
    }
  }

  pub fn filename(&mut self, filename: String) -> &mut Self {
    self.filename = Some(filename);
    self
  }

  pub async fn build(self) -> Result<Peer> {
    let id = self.id.ok_or("missing id when build FilePeer")?;
    let filename = self
      .filename
      .ok_or("missing filename when build FilePeer")?;

    let mut file = tokio::fs::OpenOptions::new()
      .create(true)
      .write(true)
      .append(true)
      .open(&filename)
      .await?;

    // writer thread
    let mut rx = self.rx;
    tokio::spawn(async move {
      loop {
        match rx.recv().await {
          Some(e) => match e {
            PeerEvent::Write(data) => {
              file
                .write_all(&data)
                .await
                .expect("FilePeer write data failed");
              file
                .write_all(b"\n")
                .await
                .expect("FilePeer write \\n failed");
              file.flush().await.expect("FilePeer flush output failed");
            }
            PeerEvent::Stop => {
              break;
            }
          },
          None => {
            break;
          }
        }
      }
    });

    Ok(Peer::new(id, self.tag, self.tx))
  }
}
