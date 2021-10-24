use bytes::Bytes;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rua::model::{Brx, Btx, NodeEvent, ReaderNode, WriterNode};
use rua::node::MockReaderNode;
use rua_macro::ReaderNode;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time;

#[derive(ReaderNode)]
pub struct RandomNode {
  btx: Btx,
  nbyte: usize,
  interval: u64, // in ms
}

impl RandomNode {
  pub fn new(buffer: usize) -> Self {
    let (btx, _) = broadcast::channel(buffer);
    Self {
      btx,
      nbyte: 8,
      interval: 200,
    }
  }

  pub fn default() -> Self {
    Self::new(16)
  }

  pub fn nbyte(mut self, n: usize) -> Self {
    self.nbyte = n;
    self
  }

  pub fn interval(mut self, ms: u64) -> Self {
    self.interval = ms;
    self
  }

  pub fn spawn(self) -> MockReaderNode {
    let interval = self.interval;
    let nbyte = self.nbyte;
    let btx = self.btx.clone();

    tokio::spawn(async move {
      loop {
        tokio::select! {
          _ = time::sleep(Duration::from_millis(interval)) => {
            if btx.send(NodeEvent::Write(random_alphanumeric_bytes(nbyte))).is_err() {
              break
            }
          }
        }
      }
    });

    MockReaderNode::new(self.btx)
  }
}

fn random_alphanumeric_bytes(n: usize) -> Bytes {
  thread_rng().sample_iter(&Alphanumeric).take(n).collect()
}
