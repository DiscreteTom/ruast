use bytes::{BufMut, BytesMut};
use tokio::{
  io::{self, AsyncReadExt, AsyncWriteExt},
  sync::mpsc::{self, Receiver, Sender},
};

use crate::model::{Peer, PeerEvent, Result};

pub struct StdioPeerBuilder {
  id: Option<u32>,
  sink: Option<Sender<PeerEvent>>,
  tag: String,

  rx: Receiver<PeerEvent>,
  tx: Sender<PeerEvent>,
}

impl StdioPeerBuilder {
  pub fn new(buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(buffer);
    Self {
      tx,
      rx,
      id: None,
      sink: None,
      tag: String::from("stdio"),
    }
  }

  pub fn id(&mut self, id: u32) -> &mut Self {
    self.id = Some(id);
    self
  }

  pub fn tag(&mut self, tag: String) -> &mut Self {
    self.tag = tag;
    self
  }

  pub fn sink(&mut self, sink: Sender<PeerEvent>) -> &mut Self {
    self.sink = Some(sink);
    self
  }

  pub fn tx(&self) -> &Sender<PeerEvent> {
    &self.tx
  }

  pub async fn build(self) -> Result<Peer> {
    let id = self.id.ok_or("missing id when build StdioPeer")?;
    let sink = self.sink.ok_or("missing sink when build StdioPeer")?;
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
                if b == b'\n'{
                  // send
                  sink
                    .send(PeerEvent::Write(buffer.freeze()))
                    .await
                    .expect("StdioPeer send event failed");
                    buffer = BytesMut::with_capacity(64);
                } else if b != b'\r'{
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
              stop_tx.send(true).await.ok();
              break;
            }
          },
          None => {
            stop_tx.send(true).await.ok();
            break;
          }
        }
      }
    });

    Ok(Peer::new(id, self.tag, self.tx))
  }
}
