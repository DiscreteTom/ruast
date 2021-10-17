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
  tag: Option<String>,
  id: Option<u32>,
  hub_tx: Option<Sender<HubEvent>>,
  disable_input: bool,
}

impl StdioPeerBuilder {
  pub fn new() -> Self {
    Self {
      tag: Some(String::from("stdio")),
      id: None,
      hub_tx: None,
      disable_input: false,
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
    let id = self.id.ok_or("id is required to build StdioPeer")?;
    let hub_tx = self
      .hub_tx
      .take()
      .ok_or("hub_tx is required to build StdioPeer")?;
    let running = Arc::new(Mutex::new(true));

    // start reader thread
    if !self.disable_input {
      let running = running.clone();
      tokio::spawn(async move {
        let stdin = std::io::stdin();
        loop {
          // read line
          let mut line = String::new();
          match stdin.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
              // remove trailing newline
              if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                  line.pop();
                }
              }
              if *running.lock().await {
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
      tag: self.tag.take().unwrap(),
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

  async fn write(&mut self, data: Bytes) -> Result<()> {
    self.stdout.write_all(&data).await?;
    self.stdout.write_all(b"\n").await?;
    Ok(self.stdout.flush().await?)
  }

  fn stop(&mut self) {
    let running = self.running.clone();
    tokio::spawn(async move {
      *running.lock().await = false;
    });
  }
}
