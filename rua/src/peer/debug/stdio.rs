use bytes::Bytes;
use rua_macro::BasicPeer;
use std::io::{self, Write};
use tokio::sync::mpsc::{self, Sender};

use crate::model::{HubEvent, Peer, PeerEvent, PeerMsg};

pub struct StdioPeerBuilder {
  tag: String,
  id: Option<i32>,
  hub_tx: Option<Sender<HubEvent>>,
  disable_input: bool,
  buffer: Option<usize>,
}

impl StdioPeerBuilder {
  pub fn new() -> Self {
    Self {
      tag: String::from("stdio"),
      id: None,
      hub_tx: None,
      disable_input: false,
      buffer: None,
    }
  }

  pub fn id(mut self, id: i32) -> Self {
    self.id = Some(id);
    self
  }

  pub fn hub_tx(mut self, tx: Sender<HubEvent>) -> Self {
    self.hub_tx = Some(tx);
    self
  }

  pub fn tag(mut self, tag: String) -> Self {
    self.tag = tag;
    self
  }

  pub fn buffer(mut self, buffer: usize) -> Self {
    self.buffer = Some(buffer);
    self
  }

  pub fn disable_input(mut self, disable: bool) -> Self {
    self.disable_input = disable;
    self
  }

  pub fn build(self) -> Box<dyn Peer> {
    Box::new(StdioPeer::new(
      self.id.expect("id is required to build StdioPeer"),
      self.tag,
      self.hub_tx.expect("hub_tx is required to build StdioPeer"),
      self.buffer.expect("buffer is required to build StdioPeer"),
      self.disable_input,
    ))
  }
}

#[derive(BasicPeer)]
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
      tokio::spawn(async move {
        let stdin = io::stdin();
        loop {
          // read line
          let mut line = String::new();
          match stdin.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
              // remove tail '\n'
              line.pop();
              // send
              hub_tx
                .send(HubEvent::PeerMsg(PeerMsg {
                  peer_id: id,
                  data: Bytes::from(line.into_bytes()),
                }))
                .await
                .unwrap()
            }
            Err(_) => break,
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
            println!("{}", String::from_utf8_lossy(&data));
            stdout.flush().unwrap();
          }
          PeerEvent::Stop => break,
        }
      }
    });

    Self { tag, id, tx }
  }
}
