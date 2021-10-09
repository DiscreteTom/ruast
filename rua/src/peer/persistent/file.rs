use rua_macro::PeerMacro;
use tokio::{
  io::AsyncWriteExt,
  sync::mpsc::{self, Sender},
};

use crate::model::{Peer, PeerEvent, Result};

pub struct FilePeerBuilder {
  tag: String,
  id: i32,
  filename: String,
  buffer: usize,
}

impl FilePeerBuilder {
  pub fn new(id: i32, filename: String, buffer: usize) -> Self {
    Self {
      tag: String::from("file"),
      id,
      filename,
      buffer,
    }
  }

  pub fn with_tag(&mut self, tag: String) -> &Self {
    self.tag = tag;
    self
  }

  pub async fn build(self) -> Result<Box<dyn Peer>> {
    Ok(Box::new(
      FilePeer::new(self.id, self.tag, self.filename, self.buffer).await?,
    ))
  }
}

#[derive(PeerMacro)]
pub struct FilePeer {
  tag: String,
  id: i32,
  tx: Sender<PeerEvent>,
}

impl FilePeer {
  async fn new(id: i32, tag: String, filename: String, buffer: usize) -> Result<Self> {
    let (tx, mut rx) = mpsc::channel(buffer);

    let mut file = tokio::fs::OpenOptions::new()
      .create(true)
      .write(true)
      .append(true)
      .open(filename)
      .await?;

    // writer thread
    tokio::spawn(async move {
      loop {
        match rx.recv().await.unwrap() {
          PeerEvent::Write(data) => {
            file.write_all(&data).await.unwrap();
            file.sync_data().await.unwrap();
          }
          PeerEvent::Stop => break,
        }
      }
    });

    Ok(Self { id, tag, tx })
  }
}
