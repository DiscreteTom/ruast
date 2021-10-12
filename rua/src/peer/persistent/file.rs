use rua_macro::BasicPeer;
use tokio::{
  io::AsyncWriteExt,
  sync::mpsc::{self, Sender},
};

use crate::model::{Peer, PeerBuilder, PeerEvent, Result};

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

  pub fn filename(mut self, filename: String) -> Self {
    self.filename = Some(filename);
    self
  }
}

impl PeerBuilder for FilePeerBuilder {
  fn id(mut self, id: u32) -> Self {
    self.id = Some(id);
    self
  }

  fn buffer(mut self, buffer: usize) -> Self {
    self.buffer = Some(buffer);
    self
  }

  fn tag(mut self, tag: String) -> Self {
    self.tag = tag;
    self
  }

  fn build(self) -> Result<Box<dyn Peer>> {
    Ok(Box::new(FilePeer::new(
      self.id.expect("id is required to build FilePeer"),
      self.tag,
      self
        .filename
        .expect("filename is required to build FilePeer"),
      self.buffer.expect("buffer is required to build FilePeer"),
    )?))
  }

  fn hub_tx(self, _: Sender<crate::model::HubEvent>) -> Self {
    self
  }
}

#[derive(BasicPeer)]
pub struct FilePeer {
  tag: String,
  id: u32,
  tx: Sender<PeerEvent>,
}

impl FilePeer {
  fn new(id: u32, tag: String, filename: String, buffer: usize) -> Result<Self> {
    let (tx, mut rx) = mpsc::channel(buffer);

    // writer thread
    tokio::spawn(async move {
      let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&filename)
        .await
        .expect(concat!(
          "open or create file failed: ",
          stringify!(filename)
        ));

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
