use bytes::Bytes;
use tokio::{io::AsyncWriteExt, sync::mpsc};

use crate::model::{Result, Rx, Tx, Urx, Utx};

pub struct FileNode {
  handle: FileNodeHandle,
  filename: Option<String>,
  rx: Rx,
  stop_rx: Urx,
}

impl FileNode {
  pub fn new(buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(buffer);
    let (stop_tx, stop_rx) = mpsc::channel(1);

    Self {
      handle: FileNodeHandle::new(tx, stop_tx),
      filename: None,
      stop_rx,
      rx,
    }
  }

  pub fn default() -> Self {
    Self::new(16)
  }

  pub fn filename(mut self, filename: String) -> Self {
    self.filename = Some(filename);
    self
  }

  pub fn handle(&self) -> FileNodeHandle {
    self.handle.clone()
  }

  pub async fn spawn(self) -> Result<FileNodeHandle> {
    let filename = self
      .filename
      .ok_or("missing filename when build FileNode")?;

    let mut file = tokio::fs::OpenOptions::new()
      .create(true)
      .write(true)
      .append(true)
      .open(&filename)
      .await?;

    let mut stop_rx = self.stop_rx;

    // writer thread
    let mut rx = self.rx;
    tokio::spawn(async move {
      loop {
        tokio::select! {
          data = rx.recv() => {
            match data {
              Some(data) => {
                file
                  .write_all(&data)
                  .await
                  .expect("FileNode write data failed");
                file
                  .write_all(b"\n")
                  .await
                  .expect("FileNode write \\n failed");
                file
                  .sync_data()
                  .await
                  .expect("FileNode flush output failed");
              }
              None => break,
            }
          }
          Some(()) = stop_rx.recv() => {
            break
          }
        }
      }
    });
    Ok(self.handle)
  }
}

#[derive(Clone)]
pub struct FileNodeHandle {
  tx: Tx,
  stop_tx: Utx,
}

impl FileNodeHandle {
  fn new(tx: Tx, stop_tx: Utx) -> Self {
    Self { tx, stop_tx }
  }

  pub fn write(&self, data: Bytes) {
    let tx = self.tx.clone();
    tokio::spawn(async move { tx.send(data).await });
  }

  pub fn stop(self) {
    let stop_tx = self.stop_tx;
    tokio::spawn(async move { stop_tx.send(()).await });
  }
}
