use bytes::Bytes;
use std::io::{self, Stdout, Write};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::model::{Event, Peer, PeerMsg, Result};

pub struct StdioPeer {
  tag: String,
  id: i32,
  hub_tx: Sender<Event>,
  tx: Sender<Bytes>,
  rx: Receiver<Bytes>,
  stdout: Stdout,
}

impl StdioPeer {
  pub fn new(id: i32, hub_tx: Sender<Event>, buffer: usize) -> Box<dyn Peer> {
    let (tx, rx) = mpsc::channel(buffer);
    Box::new(StdioPeer {
      tag: String::from("stdio"),
      id,
      hub_tx,
      tx,
      rx,
      stdout: io::stdout(),
    })
  }
}

impl Peer for StdioPeer {
  fn id(&self) -> i32 {
    self.id
  }
  fn set_tag(&mut self, tag: &str) {
    self.tag = String::from(tag);
  }
  fn tag(&self) -> &str {
    &self.tag
  }
  fn start(&mut self) -> Result<()> {
    let hub_tx = self.hub_tx.clone();
    let id = self.id;

    // start reader thread
    tokio::spawn(async move {
      let stdin = io::stdin();
      loop {
        // read line
        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() {
          break;
        } else {
          // send
          hub_tx
            .send(Event::PeerMsg(PeerMsg {
              peer_id: id,
              data: Bytes::from(line.into_bytes()),
            }))
            .await
            .unwrap()
        }
      }
    });

    // start writer thread
    let rx = self.rx;
    let stdout = self.stdout;
    tokio::spawn(async move {
      loop {
        let data = rx.recv().await.unwrap();
        print!("{}", String::from_utf8_lossy(&data));
        stdout.flush().unwrap();
      }
    });

    Ok(())
  }

  fn tx(&self) -> &Sender<Bytes> {
    &self.tx
  }
}
