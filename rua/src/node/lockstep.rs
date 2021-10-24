use bytes::Bytes;
use std::time::Duration;
use tokio::{
  sync::{broadcast, mpsc},
  time::{self, Instant},
};

use crate::model::{Brx, Btx, NodeEvent, ReaderNode, Result, Rx, Tx, WriterNode};

use super::mock::MockNode;

/// Lockstep node.
pub struct LsNode<State: Send + 'static> {
  tx: Tx,
  rx: Rx,
  btx: Btx,
  step_length_ms: u64,
  propagate_stop: bool,
  state: State,
  msg_handler: Option<Box<dyn Fn(u64, Bytes, &mut State) + 'static + Send>>,
  step_handler: Option<Box<dyn Fn(u64, &mut State) -> Bytes + 'static + Send>>,
}

impl<State: Send + 'static> ReaderNode for LsNode<State> {
  fn brx(&self) -> Brx {
    self.btx.subscribe()
  }
}

impl<State: Send + 'static> WriterNode for LsNode<State> {
  fn tx(&self) -> &Tx {
    &self.tx
  }
}

impl<State: Send + 'static> LsNode<State> {
  pub fn new(step_length_ms: u64, state: State, write_buffer: usize, read_buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(write_buffer);
    let (btx, _) = broadcast::channel(read_buffer);

    Self {
      tx,
      rx,
      btx,
      state,
      step_length_ms,
      propagate_stop: false,
      msg_handler: None,
      step_handler: None,
    }
  }

  pub fn default(step_length_ms: u64, state: State) -> Self {
    Self::new(step_length_ms, state, 16, 16)
  }

  pub fn propagate_stop(mut self, enable: bool) -> Self {
    self.propagate_stop = enable;
    self
  }

  pub fn on_msg(mut self, f: impl Fn(u64, Bytes, &mut State) + 'static + Send) -> Self {
    self.msg_handler = Some(Box::new(f));
    self
  }

  pub fn on_step(mut self, f: impl Fn(u64, &mut State) -> Bytes + 'static + Send) -> Self {
    self.step_handler = Some(Box::new(f));
    self
  }

  /// self.btx => other.tx
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

  /// other.btx => self.tx
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

  pub fn spawn(self) -> Result<MockNode> {
    let reducer = self
      .step_handler
      .ok_or("missing reducer when spawn LsNode")?;
    let mapper = self.msg_handler.ok_or("missing mapper when spawn LsNode")?;
    let mut state = self.state;
    let mut rx = self.rx;
    let btx = self.btx.clone();
    let step_length_ms = self.step_length_ms;
    let propagate_stop = self.propagate_stop;

    tokio::spawn(async move {
      let mut current = 0;
      let mut timeout = Instant::now() + Duration::from_millis(step_length_ms);

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
                    (mapper)(current, data, &mut state);
                  }
                }
              }
            }
          }
          _ = time::sleep_until(timeout) => {
            btx.send(NodeEvent::Write((reducer)(current, &mut state))).unwrap();

            current += 1;
            timeout += Duration::from_millis(step_length_ms);
          }
        }
      }
    });

    Ok(MockNode::new(self.btx, self.tx))
  }
}
