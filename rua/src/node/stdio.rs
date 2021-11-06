use bytes::{BufMut, Bytes, BytesMut};
use tokio::{
  io::{self, AsyncReadExt, AsyncWriteExt},
  sync::mpsc,
};

use crate::model::{Handle, HandleBuilder, StopRx, WriteRx};

/// StdioNode is useful to print messages to stdout.
/// If you use `on_msg` to register an stdin message handler, you may need to press Enter after you press Ctrl-C.
pub struct StdioNode {
  input_handler: Option<Box<dyn FnMut(Bytes) + Send>>,
  handle: Handle,
  rx: WriteRx,
  stop_rx: StopRx,
}

impl StdioNode {
  pub fn new(buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(buffer);
    let (stop_tx, stop_rx) = mpsc::channel(1);

    Self {
      input_handler: None,
      handle: HandleBuilder::default()
        .tx(tx)
        .stop_tx(stop_tx)
        .build()
        .unwrap(),
      rx,
      stop_rx,
    }
  }

  pub fn on_input<F>(mut self, f: F) -> Self
  where
    F: FnMut(Bytes) + Send + 'static,
  {
    self.input_handler = Some(Box::new(f));
    self
  }

  pub fn handle(&self) -> &Handle {
    &self.handle
  }

  pub fn spawn(self) -> Handle {
    let mut stop_rx = self.stop_rx;
    let mut rx = self.rx;

    // reader thread
    if let Some(mut input_handler) = self.input_handler {
      tokio::spawn(async move {
        let mut stdin = io::stdin();
        let mut buffer = BytesMut::with_capacity(64);

        loop {
          tokio::select! {
            Some(payload) = stop_rx.recv() => {
              if let Some(callback) = payload.callback {
                callback(Ok(()));
              }
              break
            }
            b = stdin.read_u8() => {
              match b {
                Ok(b) => {
                  if b == b'\n' {
                    // handle msg
                    (input_handler)(buffer.freeze());
                    // reset buffer
                    buffer = BytesMut::with_capacity(64);
                  } else if b != b'\r' {
                    // append
                    if buffer.len() == buffer.capacity() {
                      buffer.reserve(64);
                    }
                    buffer.put_u8(b);
                  }
                }
                Err(_) => break,
              }
            }
          }
        }
      });
    }

    // writer thread
    tokio::spawn(async move {
      let mut stdout = io::stdout();
      loop {
        match rx.recv().await {
          Some(payload) => {
            let result = async {
              stdout.write_all(&payload.data).await?;
              stdout.write_all(b"\n").await?;
              stdout.flush().await?;
              io::Result::Ok(())
            }
            .await;
            if let Some(callback) = payload.callback {
              if let Err(e) = result {
                callback(Err(Box::new(e)));
              } else {
                callback(Ok(()));
              }
            }
          }
          None => break,
        }
      }
    });

    self.handle
  }
}

impl Default for StdioNode {
  fn default() -> Self {
    Self::new(16)
  }
}
