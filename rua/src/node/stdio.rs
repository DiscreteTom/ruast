use bytes::{BufMut, BytesMut};
use rua_macro::{ReaderNode, WriterNode};
use tokio::{
  io::{self, AsyncReadExt, AsyncWriteExt},
  sync::{broadcast, mpsc},
};

use crate::model::{Brx, Btx, NodeEvent, ReaderNode, Rx, Tx, WriterNode};

use super::mock::{MockNode, MockWriterNode};

#[derive(ReaderNode, WriterNode)]
pub struct StdioNode {
  rx: Rx,
  tx: Tx,
  btx: Btx,
}

impl StdioNode {
  pub fn new(write_buffer: usize, read_buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(write_buffer);
    let (btx, _) = broadcast::channel(read_buffer);
    Self { tx, rx, btx }
  }

  pub fn default() -> Self {
    Self::new(16, 16)
  }

  // other.btx => self.tx
  pub fn subscribe(self, other: impl ReaderNode) -> Self {
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

  // self.btx => other.tx
  pub fn publish(self, other: impl WriterNode) -> Self {
    let mut brx = self.brx();
    let tx = other.tx().clone();

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

  pub fn echo(self) -> Self {
    let tx = self.tx().clone();
    self.publish(MockWriterNode::new(tx))
  }

  pub fn spawn(self) -> MockNode {
    let btx = self.btx.clone();
    let mut rx = self.rx;
    let (stop_tx, mut stop_rx) = mpsc::channel(1);

    // reader thread
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
                  // send
                  btx.send(NodeEvent::Write(buffer.freeze()))
                    .expect("StdioNode send event failed");
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

    // writer thread
    tokio::spawn(async move {
      let mut stdout = io::stdout();
      loop {
        match rx.recv().await {
          Some(e) => match e {
            NodeEvent::Write(data) => {
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
            NodeEvent::Stop => break,
          },
          None => break,
        }
      }
      stop_tx.send(()).await.ok();
    });

    MockNode::new(self.btx, self.tx)
  }
}
