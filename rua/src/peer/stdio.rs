use bytes::{BufMut, BytesMut};
use tokio::{
  io::{self, AsyncReadExt, AsyncWriteExt},
  sync::mpsc::{self, Receiver, Sender},
};

use crate::{impl_peer_builder, model::PeerEvent};

pub struct StdioPeer {
  sink: Option<Sender<PeerEvent>>,
  rx: Receiver<PeerEvent>,
  tx: Sender<PeerEvent>,
}

impl StdioPeer {
  impl_peer_builder!(tx, sink);

  pub fn new(buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(buffer);
    Self { tx, rx, sink: None }
  }

  pub fn spawn(self) {
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
                      .send(PeerEvent::Write(buffer.freeze()))
                      .await
                      .expect("StdioPeer send event failed");
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
            PeerEvent::Write(data) => {
              stdout
                .write_all(&data)
                .await
                .expect("StdioPeer write data failed");
              stdout
                .write_all(b"\n")
                .await
                .expect("StdioPeer write \\n failed");
              stdout.flush().await.expect("StdioPeer flush output failed");
            }
            PeerEvent::Stop => {
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
  }
}
