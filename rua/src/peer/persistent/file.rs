use tokio::{
  io::AsyncWriteExt,
  sync::mpsc::{self, Sender},
};

use crate::model::{ActivePeer, Peer, PeerEvent};

pub struct FilePeer {
  tag: String,
  id: i32,
  filename: String,
  buffer: usize,
}

impl FilePeer {
  pub fn new(id: i32, filename: String, buffer: usize) -> Self {
    FilePeer {
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

  pub fn boxed(self) -> Box<dyn Peer> {
    Box::new(self)
  }
}

impl Peer for FilePeer {
  fn id(&self) -> i32 {
    self.id
  }

  fn start(self) -> Box<dyn ActivePeer> {
    Box::new(FileActivePeer::new(
      self.id,
      self.tag,
      self.filename,
      self.buffer,
    ))
  }
}

pub struct FileActivePeer {
  tag: String,
  id: i32,
  tx: Sender<PeerEvent>,
}

impl FileActivePeer {
  pub fn new(id: i32, tag: String, filename: String, buffer: usize) -> Self {
    let (tx, mut rx) = mpsc::channel(buffer);

    // writer thread
    tokio::spawn(async move {
      let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(filename)
        .await
        .unwrap();
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

    FileActivePeer { id, tag, tx }
  }
}

impl ActivePeer for FileActivePeer {
  fn tx(&self) -> &Sender<PeerEvent> {
    &self.tx
  }
  fn id(&self) -> i32 {
    self.id
  }
  fn set_tag(&mut self, tag: String) {
    self.tag = tag;
  }
  fn tag(&self) -> &str {
    &self.tag
  }
}
