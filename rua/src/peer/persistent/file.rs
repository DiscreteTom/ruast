use bytes::Bytes;
use tokio::{
  io::AsyncWriteExt,
  sync::mpsc::{self, Receiver, Sender},
};

use crate::model::{Peer, Result};

pub struct FilePeer {
  tag: String,
  id: i32,
  filename: String,
  tx: Sender<Bytes>,
  rx: Option<Receiver<Bytes>>,
}

impl FilePeer {
  pub fn new(id: i32, filename: &str, buffer: usize) -> Result<Box<dyn Peer>> {
    let (tx, rx) = mpsc::channel(buffer);
    Ok(Box::new(FilePeer {
      tag: String::from("file"),
      id,
      tx,
      rx: Some(rx),
      filename: String::from(filename),
    }))
  }
}

impl Peer for FilePeer {
  fn tx(&self) -> &Sender<Bytes> {
    &self.tx
  }
  fn id(&self) -> i32 {
    self.id
  }
  fn set_tag(&mut self, tag: &str) {
    self.tag = String::from(tag);
  }

  fn tag(&self) -> &str {
    &self.tag
  }

  fn start(&mut self) -> Result<()> {
    let filename = self.filename.clone();
    if let Some(mut rx) = self.rx.take() {
      tokio::spawn(async move {
        let mut file = tokio::fs::OpenOptions::new()
          .create(true)
          .write(true)
          .append(true)
          .open(filename)
          .await
          .unwrap();
        loop {
          let data = rx.recv().await.unwrap();
          file.write_all(&data).await.unwrap();
          file.sync_data().await.unwrap();
        }
      });
    } else {
      panic!("stdio error")
    };
    Ok(())
  }
}
