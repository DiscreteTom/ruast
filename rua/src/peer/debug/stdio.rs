use async_trait::async_trait;
use bytes::Bytes;
use std::io::{self, Stdout, Write};
use tokio::sync::mpsc::Sender;

use crate::{
  impl_peer, impl_peer_builder,
  model::{HubEvent, Peer, PeerBuilder, PeerMsg, Result},
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
  impl_peer_builder!(all);

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
}

pub struct StdioPeer {
  tag: String,
  id: u32,
  stdout: Stdout,
}

impl StdioPeer {
  fn new(
    id: u32,
    tag: String,
    hub_tx: Sender<HubEvent>,
    buffer: usize,
    disable_input: bool,
  ) -> Self {
    // start reader thread
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

    Self {
      tag,
      id,
      stdout: io::stdout(),
    }
  }
}

#[async_trait]
impl Peer for StdioPeer {
  impl_peer!(all);

  async fn write(&self, data: Bytes) -> Result<()> {
    println!("{}", String::from_utf8_lossy(&data));
    Ok(self.stdout.flush()?)
  }

  fn stop(&mut self) {}
}
