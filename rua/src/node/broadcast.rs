use bytes::Bytes;
use rua_macro::{Stoppable, Writable, WritableStoppable};
use tokio::sync::mpsc::{self, Sender};

use crate::model::{Error, Result, Rx, Stoppable, Tx, Utx, Writable, WritableStoppable};

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

#[derive(Clone, Writable, Stoppable, WritableStoppable)]
pub struct StoppableBcNode {
  tx: Tx,
  stop_tx: Utx,
  node_tx: Sender<Box<dyn WritableStoppable + Send>>,
}

impl StoppableBcNode {
  pub fn new(buffer: usize) -> Self {
    let (tx, mut rx): (Tx, Rx) = mpsc::channel(buffer);
    let (stop_tx, mut stop_rx) = mpsc::channel(1);
    let (node_tx, mut node_rx) = mpsc::channel(1);

    tokio::spawn(async move {
      let mut targets: Vec<Box<dyn WritableStoppable + Send>> = Vec::new();

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
          Some(()) = stop_rx.recv() => {
            for node in targets {
              node.stop();
            }
            break
          }
        }
      }
    });

    Self {
      tx,
      node_tx,
      stop_tx,
    }
  }

  pub fn default() -> Self {
    Self::new(16)
  }

  pub fn add_target(&mut self, t: impl WritableStoppable + 'static + Send) {
    let node_tx = self.node_tx.clone();
    tokio::spawn(async move { node_tx.send(Box::new(t)).await });
  }
}
