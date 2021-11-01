use bytes::{BufMut, Bytes, BytesMut};
use tokio::{
  io::{self, AsyncReadExt, AsyncWriteExt},
  sync::mpsc,
};

use crate::model::{Rx, Urx, WritableStoppableHandle};

/// StdioNode is useful to print messages to stdout.
/// If you use `on_msg` to register an stdin message handler, you may need to press Enter after you press Ctrl-C.
pub struct StdioNode {
  msg_handler: Option<Box<dyn FnMut(Bytes) + Send>>,
  handle: WritableStoppableHandle,
  rx: Rx,
  stop_rx: Urx,
}

impl StdioNode {
  pub fn new(buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(buffer);
    let (stop_tx, stop_rx) = mpsc::channel(1);

    Self {
      msg_handler: None,
      handle: WritableStoppableHandle::new(tx, stop_tx),
      rx,
      stop_rx,
    }
  }

  pub fn default() -> Self {
    Self::new(16)
  }

  pub fn on_msg(mut self, f: impl FnMut(Bytes) + 'static + Send) -> Self {
    self.msg_handler = Some(Box::new(f));
    self
  }

  pub fn handle(&self) -> WritableStoppableHandle {
    self.handle.clone()
  }

  pub fn spawn(self) -> WritableStoppableHandle {
    let mut stop_rx = self.stop_rx;
    let mut rx = self.rx;

    // reader thread
    if let Some(mut msg_handler) = self.msg_handler {
      tokio::spawn(async move {
        let mut stdin = io::stdin();
        let mut buffer = BytesMut::with_capacity(64);

        loop {
          tokio::select! {
            Some(()) = stop_rx.recv() => {
              break
            }
            b = stdin.read_u8() => {
              match b {
                Ok(b) => {
                  if b == b'\n' {
                    // handle msg
                    (msg_handler)(buffer.freeze());
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
          Some(data) => {
            stdout
              .write_all(&data)
              .await
              .expect("StdioNode write data failed");
            stdout
              .write_all(b"\n")
              .await
              .expect("StdioNode write \\n failed");
            stdout.flush().await.expect("StdioNode flush output failed");
          }
          None => break,
        }
      }
    });

    self.handle
  }
}
