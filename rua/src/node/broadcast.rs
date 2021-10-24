use rua_macro::{ReaderNode, WriterNode};
use tokio::sync::{broadcast, mpsc};

use crate::model::{Brx, Btx, NodeEvent, ReaderNode, Rx, Tx, WriterNode};

use super::mock::MockNode;

/// Broadcast node.
#[derive(ReaderNode, WriterNode)]
pub struct BcNode {
  tx: Tx,
  rx: Rx,
  btx: Btx,
  propagate_stop: bool,
}

impl BcNode {
  pub fn new(write_buffer: usize, read_buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(write_buffer);
    let (btx, _) = broadcast::channel(read_buffer);
    Self {
      tx,
      rx,
      btx,
      propagate_stop: false,
    }
  }

  pub fn default() -> Self {
    Self::new(16, 16)
  }

  pub fn propagate_stop(mut self, enable: bool) -> Self {
    self.propagate_stop = enable;
    self
  }

  // other.btx => self.tx
  pub fn subscribe(self, other: &impl ReaderNode) -> Self {
    let mut brx = other.brx();
    let tx = self.tx().clone();
    tokio::spawn(async move {
      loop {
        match brx.recv().await {
          Ok(e) => {
            if tx.send(e).await.is_err() {
              break;
            }
          }
          Err(_) => break,
        }
      }
    });
    self
  }

  // self.btx => other.tx
  pub fn publish(self, other: &impl WriterNode) -> Self {
    let mut brx = self.brx();
    let tx = other.tx().clone();

    tokio::spawn(async move {
      loop {
        match brx.recv().await {
          Ok(e) => {
            if tx.send(e).await.is_err() {
              break;
            }
          }
          Err(_) => break,
        }
      }
    });
    self
  }

  pub fn spawn(self) -> MockNode {
    let btx = self.btx.clone();
    let mut rx = self.rx;
    let propagate_stop = self.propagate_stop;

    tokio::spawn(async move {
      loop {
        match rx.recv().await {
          Some(e) => match e {
            NodeEvent::Write(data) => {
              btx.send(NodeEvent::Write(data)).unwrap();
            }
            NodeEvent::Stop => {
              if propagate_stop {
                btx.send(NodeEvent::Stop).unwrap();
              }
              break;
            }
          },
          None => break,
        }
      }
    });

    MockNode::new(self.btx, self.tx)
  }
}
