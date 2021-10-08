use bytes::Bytes;
use std::io::{self, Write};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::model::{HubEvent, Peer, PeerEvent, PeerMsg, Result};

pub struct StdioPeer {
  tag: String,
  id: i32,
  hub_tx: Sender<HubEvent>,
  tx: Sender<PeerEvent>,
  rx: Option<Receiver<PeerEvent>>,
}

impl StdioPeer {
  pub fn new(id: i32, hub_tx: Sender<HubEvent>, buffer: usize) -> Box<dyn Peer> {
    let (tx, rx) = mpsc::channel(buffer);
    Box::new(StdioPeer {
      tag: String::from("stdio"),
      id,
      hub_tx,
      tx,
      rx: Some(rx),
    })
  }
}

impl Peer for StdioPeer {
  fn tx(&self) -> &Sender<PeerEvent> {
    &self.tx
  }
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
    // start reader thread
    let hub_tx = self.hub_tx.clone();
    let id = self.id;
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
            .send(HubEvent::PeerMsg(PeerMsg {
              peer_id: id,
              data: Bytes::from(line.into_bytes()),
            }))
            .await
            .unwrap()
        }
      }
    });

    // start writer thread
    if let Some(mut rx) = self.rx.take() {
      tokio::spawn(async move {
        let mut stdout = io::stdout();
        loop {
          match rx.recv().await.unwrap() {
            PeerEvent::Write(data) => {
              print!("{}", String::from_utf8_lossy(&data));
              stdout.flush().unwrap();
            }
            PeerEvent::Stop => break,
          }
        }
      });
    } else {
      panic!("stdio error")
    }

    Ok(())
  }
}
