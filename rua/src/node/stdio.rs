use bytes::{BufMut, Bytes, BytesMut};
use tokio::{
  io::{self, AsyncReadExt, AsyncWriteExt},
  sync::mpsc,
};

use crate::{
  go,
  model::{Handle, HandleBuilder, StopRx, WriteRx},
  take_mut,
};

/// StdioNode is useful to print messages to stdout.
/// If you use `on_input` to register an stdin message handler, you may need to press Enter after you press Ctrl-C.
pub struct StdioNode {
  input_handler: Option<Box<dyn FnMut(Bytes) + Send>>,
  handle: Handle,
  rx: WriteRx,
  stop_rx: StopRx,
}

impl Default for StdioNode {
  fn default() -> Self {
    Self::new(16)
  }
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
    take_mut!(self, stop_rx, rx);
    let (reader_stop_tx, mut reader_stop_rx) = mpsc::channel(1);
    let (writer_stop_tx, mut writer_stop_rx) = mpsc::channel(1);

    // stopper thread
    go! {
      if let Some(payload) = stop_rx.recv().await {
        reader_stop_tx.send(()).await.ok();
        writer_stop_tx.send(()).await.ok();
        (payload.callback)(Ok(()));
      }
      // else, all stop_tx are dropped, stop_rx is disabled

      // stop_rx is dropped, later stop_tx.send will throw ChannelClosed error.
    };

    // reader thread
    if let Some(mut input_handler) = self.input_handler {
      go! {
        let mut stdin = io::stdin();
        let mut buffer = BytesMut::with_capacity(64);

        loop {
          tokio::select! {
            Some(()) = reader_stop_rx.recv() => {
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
                Err(_) => break, // stdin error
              }
            }
          }
        }
      };
    }

    // writer thread
    go! {
      let mut stdout = io::stdout();
      loop {
        tokio::select! {
          Some(()) = writer_stop_rx.recv() => {
            break
          }
          payload = rx.recv() => {
            if let Some(payload) = payload {
              let result = async {
                stdout.write_all(&payload.data).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
                io::Result::Ok(())
              }
              .await;
              if let Err(e) = result {
                (payload.callback)(Err(Box::new(e)));
                break
              } else {
                (payload.callback)(Ok(()));
              }
            } else {
              break // all tx are dropped
            }
          }
        }
      }
    };

    self.handle
  }
}
