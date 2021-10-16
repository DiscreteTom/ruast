use async_trait::async_trait;
use bytes::Bytes;
use std::sync::Arc;
use tokio::{
  io::{self, AsyncWriteExt, Stdout},
  sync::{mpsc::Sender, Mutex},
};

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

#[async_trait]
impl PeerBuilder for StdioPeerBuilder {
  impl_peer_builder!(all);

  async fn build(&mut self) -> Result<Box<dyn Peer>> {
    let id = self.id.expect("id is required to build StdioPeer");
    let tag = self.tag.clone();
    let hub_tx = self
      .hub_tx
      .as_ref()
      .expect("hub_tx is required to build StdioPeer")
      .clone();
    let buffer = self.buffer.expect("buffer is required to build StdioPeer");
    let running = Arc::new(Mutex::new(true));

    // start reader thread
    if !self.disable_input {
      tokio::spawn(async move {
        let stdin = std::io::stdin();
        loop {
          // read line
          let mut line = String::new();
          match stdin.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
              if *running.lock().await {
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
              } else {
                break;
              }
            }
            Err(_) => break,
          }
        }
      });
    }

    Ok(Box::new(StdioPeer {
      id,
      tag,
      stdout: io::stdout(),
      running: running.clone(),
    }))
  }
}

pub struct StdioPeer {
  tag: String,
  id: u32,
  stdout: Stdout,
  running: Arc<Mutex<bool>>,
}

#[async_trait]
impl Peer for StdioPeer {
  impl_peer!(all);

  async fn write(&self, data: Bytes) -> Result<()> {
    self.stdout.write_all(&data.to_vec()).await?;
    println!("{}", String::from_utf8_lossy(&data));
    Ok(self.stdout.flush().await?)
  }

  fn stop(&mut self) {
    let running = self.running.clone();
    tokio::spawn(async move {
      *running.lock().await = false;
    });
  }
}
