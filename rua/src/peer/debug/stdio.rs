use bytes::Bytes;
use std::io::{self, Write};
use tokio::sync::mpsc::{self, Sender};

use crate::{
  model::{HubEvent, Peer, PeerBuilder, PeerMsg, Result},
  BasicPeer,
};

pub struct StdioPeerBuilder {
  tag: String,
  id: Option<u32>,
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

  pub fn disable_input(mut self, disable: bool) -> Self {
    self.disable_input = disable;
    self
  }

  pub fn boxed(self) -> Box<dyn PeerBuilder> {
    Box::new(self)
  }
}

impl PeerBuilder for StdioPeerBuilder {
  fn id(&mut self, id: u32) -> &mut dyn PeerBuilder {
    self.id = Some(id);
    self
  }

  fn hub_tx(&mut self, tx: Sender<HubEvent>) -> &mut dyn PeerBuilder {
    self.hub_tx = Some(tx);
    self
  }

  fn tag(&mut self, tag: String) -> &mut dyn PeerBuilder {
    self.tag = tag;
    self
  }

  fn buffer(&mut self, buffer: usize) -> &mut dyn PeerBuilder {
    self.buffer = Some(buffer);
    self
  }

  fn build(&mut self) -> Result<Box<dyn Peer>> {
    Ok(Box::new(StdioPeer::new(
      self.id.expect("id is required to build StdioPeer"),
      self.tag.clone(),
      self
        .hub_tx
        .as_ref()
        .expect("hub_tx is required to build StdioPeer")
        .clone(),
      self.buffer.expect("buffer is required to build StdioPeer"),
      self.disable_input,
    )))
  }

  fn get_id(&self) -> Option<u32> {
    self.id
  }

  fn get_tag(&self) -> &str {
    &self.tag
  }
}

pub struct StdioPeer {
  tag: String,
  id: u32,
}

impl StdioPeer {
  fn new(
    id: u32,
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

impl Peer for StdioPeer {
  BasicPeer!();
}
