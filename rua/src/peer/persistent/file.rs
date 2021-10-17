use crate::{
  impl_peer, impl_peer_builder,
  model::HubEvent,
  model::{Peer, PeerBuilder, Result},
};

use async_trait::async_trait;
use bytes::Bytes;
use tokio::fs::File;
use tokio::{io::AsyncWriteExt, sync::mpsc::Sender};

pub struct FilePeerBuilder {
  tag: Option<String>,
  id: Option<u32>,
  filename: Option<String>,
  hub_tx: Option<Sender<HubEvent>>,
  separator: Option<Bytes>,
  transformer: Option<Box<dyn Fn(Bytes) -> Bytes + 'static + Send + Sync>>,
}

impl FilePeerBuilder {
  pub fn new() -> Self {
    Self {
      tag: Some(String::from("file")),
      id: None,
      filename: None,
      hub_tx: None,
      separator: Some(Bytes::from_static(b"")),
      transformer: Some(Box::new(|data| data)),
    }
  }

  pub fn filename(mut self, filename: String) -> Self {
    self.filename = Some(filename);
    self
  }

  pub fn separator(mut self, s: &'static str) -> Self {
    self.separator = Some(Bytes::from(s));
    self
  }

  pub fn binary_separator(mut self, s: Bytes) -> Self {
    self.separator = Some(s);
    self
  }

  pub fn transformer<F>(mut self, f: F) -> Self
  where
    F: Fn(Bytes) -> Bytes + 'static + Send + Sync,
  {
    self.transformer = Some(Box::new(f));
    self
  }

  pub fn boxed(self) -> Box<dyn PeerBuilder> {
    Box::new(self)
  }
}

#[async_trait]
impl PeerBuilder for FilePeerBuilder {
  impl_peer_builder!(all);

  async fn build(&mut self) -> Result<Box<dyn Peer>> {
    let id = self.id.ok_or("id is required to build FilePeer")?;
    let filename = self
      .filename
      .take()
      .ok_or("filename is required to build FilePeer")?;

    let file = tokio::fs::OpenOptions::new()
      .create(true)
      .write(true)
      .append(true)
      .open(&filename)
      .await?;

    Ok(Box::new(FilePeer {
      tag: self.tag.take().unwrap(),
      id,
      file,
      transformer: self.transformer.take().unwrap(),
      separator: self.separator.take().unwrap(),
    }))
  }
}

pub struct FilePeer {
  tag: String,
  id: u32,
  file: File,
  separator: Bytes,
  transformer: Box<dyn Fn(Bytes) -> Bytes + 'static + Send + Sync>,
}

#[async_trait]
impl Peer for FilePeer {
  impl_peer!(all);

  async fn write(&mut self, data: Bytes) -> Result<()> {
    self.file.write_all(&(self.transformer)(data)).await?;
    self.file.write_all(&self.separator).await?;
    Ok(self.file.sync_data().await?)
  }

  fn stop(&mut self) {}
}
