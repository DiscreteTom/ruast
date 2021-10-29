use bytes::Bytes;
use tokio::sync::mpsc::{self, Sender};

use crate::model::{Rx, Tx, Writable};

pub struct BcNode {
  tx: Tx,
  node_tx: Sender<Box<dyn Writable + Send>>,
}

impl BcNode {
  pub fn new(buffer: usize) -> Self {
    let (tx, mut rx): (Tx, Rx) = mpsc::channel(buffer);
    let (node_tx, mut node_rx) = mpsc::channel(1);

    tokio::spawn(async move {
      let mut targets: Vec<Box<dyn Writable + Send>> = Vec::new();

      loop {
        tokio::select! {
          data = rx.recv() => {
            match data {
              Some(data) => {
                for node in &targets {
                  node.write(data.clone());
                }
              }
              None => break
            }
          }
          Some(node) = node_rx.recv() => {
            targets.push(node);
          }
        }
      }
    });

    Self { tx, node_tx }
  }

  pub fn default() -> Self {
    Self::new(16)
  }

  pub fn add_target(&mut self, t: impl Writable + 'static + Send) {
    let node_tx = self.node_tx.clone();
    tokio::spawn(async move { node_tx.send(Box::new(t)).await });
  }
}

impl Writable for BcNode {
  fn write(&self, data: Bytes) {
    let tx = self.tx.clone();
    tokio::spawn(async move { tx.send(data).await });
  }
}
