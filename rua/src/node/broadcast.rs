use bytes::Bytes;
use rua_macro::Writable;
use tokio::sync::mpsc::{self, Sender};

use crate::model::{Error, Result, Rx, Tx, Writable};

#[derive(Clone, Writable)]
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
                let mut dead_nodes = Vec::new();
                for (i, node) in targets.iter().enumerate() {
                  if node.write(data.clone()).is_err() {
                    dead_nodes.push(i);
                  }
                }
                // clean dead nodes
                while let Some(i) = dead_nodes.pop() {
                  targets.remove(i);
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

// Stoppable Bc
