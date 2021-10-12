use rua_macro::BasicPeer;
use tokio::{
  io::AsyncWriteExt,
  sync::mpsc::{self, Sender},
};

use crate::model::{Peer, PeerEvent, Result};

pub struct FilePeerBuilder {
  tag: String,
  id: Option<u32>,
  filename: Option<String>,
  buffer: Option<usize>,
}

impl FilePeerBuilder {
  pub fn new() -> Self {
    Self {
      tag: String::from("file"),
      id: None,
      filename: None,
      buffer: None,
    }
  }

  pub fn id(mut self, id: u32) -> Self {
    self.id = Some(id);
    self
  }
  pub fn filename(mut self, filename: String) -> Self {
    self.filename = Some(filename);
    self
  }
  pub fn buffer(mut self, buffer: usize) -> Self {
    self.buffer = Some(buffer);
    self
  }

  pub fn tag(mut self, tag: String) -> Self {
    self.tag = tag;
    self
  }

  pub async fn build(self) -> Result<Box<dyn Peer>> {
    Ok(Box::new(
      FilePeer::new(
        self.id.unwrap(),
        self.tag,
        self.filename.unwrap(),
        self.buffer.unwrap(),
      )
      .await?,
    ))
  }
}

#[derive(BasicPeer)]
pub struct FilePeer {
  tag: String,
  id: u32,
  tx: Sender<PeerEvent>,
}

impl FilePeer {
  async fn new(id: u32, tag: String, filename: String, buffer: usize) -> Result<Self> {
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
