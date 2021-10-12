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

  pub fn boxed(self) -> Box<dyn PeerBuilder> {
    Box::new(self)
  }
}

impl PeerBuilder for FilePeerBuilder {
  fn id(&mut self, id: u32) -> &mut dyn PeerBuilder {
    self.id = Some(id);
    self
  }

  fn buffer(&mut self, buffer: usize) -> &mut dyn PeerBuilder {
    self.buffer = Some(buffer);
    self
  }

  fn tag(&mut self, tag: String) -> &mut dyn PeerBuilder {
    self.tag = tag;
    self
  }

  fn hub_tx(&mut self, _: Sender<crate::model::HubEvent>) -> &mut dyn PeerBuilder {
    self
  }

  fn build(&mut self) -> Result<Box<dyn Peer>> {
    Ok(Box::new(FilePeer::new(
      self.id.expect("id is required to build FilePeer"),
      self.tag.clone(),
      self
        .filename
        .as_ref()
        .expect("filename is required to build FilePeer")
        .clone(),
      self.buffer.expect("buffer is required to build FilePeer"),
    )?))
  }

  fn get_id(&self) -> Option<u32> {
    self.id
  }

  fn get_tag(&self) -> &str {
    &self.tag
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
