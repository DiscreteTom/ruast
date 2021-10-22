use tokio::{
  io::AsyncWriteExt,
  sync::mpsc::{self, Receiver, Sender},
};

use crate::{
  impl_peer_builder,
  model::{NodeEvent, Result},
};

pub struct FilePeer {
  rx: Receiver<NodeEvent>,
  tx: Sender<NodeEvent>,
  filename: Option<String>,
}

impl FilePeer {
  impl_peer_builder!(tx);

  pub fn new(buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(buffer);

    Self {
      tx,
      rx,
      filename: None,
    }
  }

  pub fn filename(mut self, filename: String) -> Self {
    self.filename = Some(filename);
    self
  }

  pub async fn spawn(self) -> Result<Sender<NodeEvent>> {
    let filename = self
      .filename
      .ok_or("missing filename when build FilePeer")?;

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
                .expect("FilePeer write data failed");
              file
                .write_all(b"\n")
                .await
                .expect("FilePeer write \\n failed");
              file
                .sync_data()
                .await
                .expect("FilePeer flush output failed");
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

    Ok(self.tx)
  }
}
