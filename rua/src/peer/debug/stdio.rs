use bytes::Bytes;
use std::io::{self, Write};
use tokio::sync::mpsc::{self, Sender};

use crate::model::{HubEvent, Peer, PeerEvent, PeerMsg};

pub struct StdioPeerBuilder {
  tag: String,
  id: i32,
  hub_tx: Sender<HubEvent>,
  disable_input: bool,
  buffer: usize,
}

impl StdioPeerBuilder {
  pub fn new(id: i32, hub_tx: Sender<HubEvent>, buffer: usize) -> Self {
    Self {
      tag: String::from("stdio"),
      id,
      hub_tx,
      disable_input: false,
      buffer,
    }
  }

  pub fn disable_input(&mut self, disable: bool) -> &Self {
    self.disable_input = disable;
    self
  }

  pub fn with_tag(&mut self, tag: String) -> &Self {
    self.tag = tag;
    self
  }

  pub fn build(self) -> Box<dyn Peer> {
    Box::new(StdioPeer::new(
      self.id,
      self.tag,
      self.hub_tx,
      self.buffer,
      self.disable_input,
    ))
  }
}

pub struct StdioPeer {
  tag: String,
  id: i32,
  tx: Sender<PeerEvent>,
}

impl StdioPeer {
  fn new(
    id: i32,
    tag: String,
    hub_tx: Sender<HubEvent>,
    buffer: usize,
    disable_input: bool,
  ) -> Self {
    let (tx, mut rx) = mpsc::channel(buffer);

    // reader thread
    if !disable_input {
      let id = id;
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
    }

    // writer thread
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

    Self { tag, id, tx }
  }
}

impl Peer for StdioPeer {
  fn tx(&self) -> &Sender<PeerEvent> {
    &self.tx
  }
  fn id(&self) -> i32 {
    self.id
  }
  fn set_tag(&mut self, tag: String) {
    self.tag = tag;
  }
  fn tag(&self) -> &str {
    &self.tag
  }
}
