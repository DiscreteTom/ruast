use tokio::{io::AsyncWriteExt, sync::mpsc};

use crate::model::{GeneralResult, Handle, HandleBuilder, StopRx, WriteRx};

pub struct FileNode<'a> {
  handle: Handle,
  filename: Option<&'a str>,
  rx: WriteRx,
  stop_rx: StopRx,
}

impl<'a> Default for FileNode<'a> {
  fn default() -> Self {
    Self::new(16)
  }
}

impl<'a> FileNode<'a> {
  pub fn new(buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(buffer);
    let (stop_tx, stop_rx) = mpsc::channel(1);

    Self {
      handle: HandleBuilder::default()
        .tx(tx)
        .stop_tx(stop_tx)
        .build()
        .unwrap(),
      filename: None,
      stop_rx,
      rx,
    }
  }

  pub fn filename(mut self, filename: &'a str) -> Self {
    self.filename = Some(filename);
    self
  }

  pub fn handle(&self) -> &Handle {
    &self.handle
  }

  pub async fn spawn(self) -> GeneralResult<Handle> {
    let filename = self
      .filename
      .ok_or("missing filename when build FileNode")?;

    let mut file = tokio::fs::OpenOptions::new()
      .create(true)
      .write(true)
      .append(true)
      .open(filename)
      .await?;

    // writer thread
    let mut stop_rx = self.stop_rx;
    let mut rx = self.rx;
    tokio::spawn(async move {
      loop {
        tokio::select! {
          Some(payload) = stop_rx.recv() => {
            (payload.callback)(Ok(()));
            break
          }
          payload = rx.recv() => {
            if let Some(payload) = payload {
              let result = async {
                file.write_all(&payload.data).await?;
                file.write_all(b"\n").await?;
                file.sync_data().await?;
                std::io::Result::Ok(())
              }.await;
              if let Err(e) = result {
                (payload.callback)(Err(Box::new(e)));
              } else {
                (payload.callback)(Ok(()));
              }
            } else {
              break // all tx are dropped
            }
          }
        }
      }
    });
    Ok(self.handle)
  }
}
