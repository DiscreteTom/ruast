use bytes::{BufMut, Bytes, BytesMut};
use tokio::{
  io::{self, AsyncReadExt, AsyncWriteExt},
  sync::mpsc,
};

use crate::model::{Rx, Tx, Urx, Utx};

pub struct StdioNode {
  msg_handler: Option<Box<dyn Fn(Bytes) + Send>>,
  handle: StdioNodeHandle,
  rx: Rx,
  stop_rx: Urx,
}

impl StdioNode {
  pub fn new(buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(buffer);
    let (stop_tx, stop_rx) = mpsc::channel(1);

    Self {
      msg_handler: None,
      handle: StdioNodeHandle::new(tx, stop_tx),
      rx,
      stop_rx,
    }
  }

  pub fn default() -> Self {
    Self::new(16)
  }

  pub fn on_msg(mut self, f: impl Fn(Bytes) + 'static + Send) -> Self {
    self.msg_handler = Some(Box::new(f));
    self
  }

  pub fn handle(&self) -> StdioNodeHandle {
    self.handle.clone()
  }

  pub fn spawn(self) -> StdioNodeHandle {
    let mut stop_rx = self.stop_rx;
    let mut rx = self.rx;

    // reader thread
    if let Some(msg_handler) = self.msg_handler {
      tokio::spawn(async move {
        let mut stdin = io::stdin();
        let mut buffer = BytesMut::with_capacity(64);

        loop {
          tokio::select! {
            _ = stop_rx.recv() => {
              break
            }
            b = stdin.read_u8() => {
              match b{
                Ok(b)=>{
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
                Err(_)=>{
                  break;
                }
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

#[derive(Clone)]
pub struct StdioNodeHandle {
  tx: Tx,
  stop_tx: Utx,
}

impl StdioNodeHandle {
  fn new(tx: Tx, stop_tx: Utx) -> Self {
    Self { tx, stop_tx }
  }

  pub fn write(&self, data: Bytes) {
    let tx = self.tx.clone();
    tokio::spawn(async move { tx.send(data).await });
  }

  pub fn stop(self) {
    let stop_tx = self.stop_tx.clone();
    tokio::spawn(async move { stop_tx.send(()).await });
  }
}
