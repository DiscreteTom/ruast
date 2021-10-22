use bytes::{BufMut, BytesMut};
use tokio::{
  io::{self, AsyncReadExt, AsyncWriteExt},
  sync::mpsc,
};

use crate::{
  impl_node,
  model::{NodeEvent, Rx, Tx},
};

pub struct StdioNode {
  sink: Option<Tx>,
  rx: Rx,
  tx: Tx,
}

impl StdioNode {
  impl_node!(tx, sink);

  pub fn new(buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(buffer);
    Self { tx, rx, sink: None }
  }

  pub fn echo(mut self) -> Self {
    self.sink = Some(self.tx.clone());
    self
  }

  pub fn spawn(self) -> Tx {
    let mut rx = self.rx;
    let (stop_tx, mut stop_rx) = mpsc::channel(1);

    // reader thread
    if let Some(sink) = self.sink {
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
                    sink
                      .send(NodeEvent::Write(buffer.freeze()))
                      .await
                      .expect("StdioNode send event failed");
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
            NodeEvent::Stop => {
              stop_tx.send(()).await.ok();
              break;
            }
          },
          None => {
            stop_tx.send(()).await.ok();
            break;
          }
        }
      }
    });

    self.tx
  }
}
