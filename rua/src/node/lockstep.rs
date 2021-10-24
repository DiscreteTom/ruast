use bytes::{Bytes, BytesMut};
use rua_macro::{ReaderNode, WriterNode};
use std::time::Duration;
use tokio::{
  sync::{broadcast, mpsc},
  time::{self, Instant},
};

use crate::model::{Brx, Btx, NodeEvent, ReaderNode, Rx, Tx, WriterNode};

use super::mock::MockNode;

/// Lockstep node.
#[derive(ReaderNode, WriterNode)]
pub struct LsNode {
  tx: Tx,
  rx: Rx,
  btx: Btx,
  step_length_ms: u64,
  propagate_stop: bool,
  reducer: Box<dyn Fn(u64, Vec<Bytes>) -> Bytes + 'static + Send>,
}

impl LsNode {
  pub fn new(step_length_ms: u64, write_buffer: usize, read_buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(write_buffer);
    let (btx, _) = broadcast::channel(read_buffer);

    Self {
      tx,
      rx,
      btx,
      step_length_ms,
      propagate_stop: false,
      reducer: Box::new(|_, data| {
        let mut buffer = BytesMut::new();
        for d in data {
          buffer.extend_from_slice(&d);
        }
        buffer.freeze()
      }),
    }
  }

  pub fn default_with_step_length(step_length_ms: u64) -> Self {
    Self::new(step_length_ms, 16, 16)
  }

  pub fn propagate_stop(mut self, enable: bool) -> Self {
    self.propagate_stop = enable;
    self
  }

  pub fn reducer(mut self, f: impl Fn(u64, Vec<Bytes>) -> Bytes + 'static + Send) -> Self {
    self.reducer = Box::new(f);
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
    let mut rx = self.rx;
    let btx = self.btx.clone();
    let step_length_ms = self.step_length_ms;
    let propagate_stop = self.propagate_stop;
    let reducer = self.reducer;

    tokio::spawn(async move {
      let mut current = 0;
      let mut timeout = Instant::now() + Duration::from_millis(step_length_ms);
      let mut buffer = Vec::new();

      loop {
        tokio::select! {
          e = rx.recv() => {
            match e {
              None => break,
              Some(e) => {
                match e {
                  NodeEvent::Stop => {
                    if propagate_stop {
                      btx.send(NodeEvent::Stop).unwrap();
                    }
                    break
                  }
                  NodeEvent::Write(data) => {
                    buffer.push(data);
                  }
                }
              }
            }
          }
          _ = time::sleep_until(timeout) => {
            btx.send(NodeEvent::Write((reducer)(current, buffer))).unwrap();
            buffer = Vec::new(); // reset buffer

            current += 1;
            timeout += Duration::from_millis(step_length_ms);
          }
        }
      }
    });

    MockNode::new(self.btx, self.tx)
  }
}
