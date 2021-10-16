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
  tag: String,
  id: Option<u32>,
  filename: Option<String>,
  hub_tx: Option<Sender<HubEvent>>,
}

impl FilePeerBuilder {
  pub fn new() -> Self {
    Self {
      tag: String::from("file"),
      id: None,
      filename: None,
      hub_tx: None,
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
      tag: self.tag.clone(),
      id,
      file,
    }))
  }
}

pub struct FilePeer {
  tag: String,
  id: u32,
  file: File,
}

#[async_trait]
impl Peer for FilePeer {
  impl_peer!(all);

  async fn write(&mut self, data: Bytes) -> Result<()> {
    self.file.write_all(&data).await?;
    Ok(self.file.sync_data().await?)
  }

  fn stop(&mut self) {}
}
