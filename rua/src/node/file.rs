use crate::model::{NodeEvent, ReaderNode, Result, Rx, Tx, WriterNode};
use rua_macro::WriterNode;
use tokio::{io::AsyncWriteExt, sync::mpsc};

use super::mock::MockWriterNode;

#[derive(WriterNode)]
pub struct FileNode {
  rx: Rx,
  tx: Tx,
  filename: Option<String>,
}

impl FileNode {
  pub fn new(buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(buffer);

    Self {
      tx,
      rx,
      filename: None,
    }
  }

  pub fn default() -> Self {
    Self::new(16)
  }

  pub fn filename(mut self, filename: String) -> Self {
    self.filename = Some(filename);
    self
  }

  pub fn subscribe(self, other: &impl ReaderNode) -> Self {
    let mut brx = other.brx();
    let tx = self.tx().clone();
    tokio::spawn(async move {
      loop {
        match brx.recv().await {
          Ok(e) => {
            if tx.send(e).await.is_err() {
              break;
            }
          }
          Err(_) => break,
        }
      }
    });
    self
  }

  pub async fn spawn(self) -> Result<MockWriterNode> {
    let filename = self
      .filename
      .ok_or("missing filename when build FileNode")?;

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
            NodeEvent::Write(data) => {
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
            NodeEvent::Stop => {
              break;
            }
          },
          None => {
            break;
          }
        }
      }
    });

    Ok(MockWriterNode::new(self.tx))
  }
}
