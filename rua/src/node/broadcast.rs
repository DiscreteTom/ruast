use std::sync::Arc;

use bytes::Bytes;
use tokio::sync::{
  mpsc::{self, Sender},
  Mutex,
};

use crate::model::{GeneralResult, Handle, HandleBuilder, StopTx, WriteRx, WriteTx};

#[derive(Clone, Default)]
pub struct Broadcaster {
  timeout_ms: Option<u64>,
  targets: Arc<Mutex<Vec<Handle>>>,
  propagate_stop: bool,
}

impl Broadcaster {
  pub fn propagate_stop(mut self, enable: bool) -> Self {
    self.propagate_stop = enable;
    self
  }

  pub fn add_target(&self, handle: Handle) {
    let targets = self.targets.clone();
    tokio::spawn(async move { targets.lock().await.push(handle) });
  }

  pub fn add_target_then<F>(&self, handle: Handle, callback: F)
  where
    F: Fn() + Send + Sync + 'static,
  {
    let targets = self.targets.clone();
    tokio::spawn(async move {
      targets.lock().await.push(handle);
      callback()
    });
  }

  /// Write will be canceled if timeout, in this case you may need to increase the node's buffer.
  pub fn write(&self, data: Bytes) {
    let targets = self.targets.clone();
    Self::inner_write(targets, data, self.timeout_ms, |_| {})
  }

  /// Write will be canceled if timeout, in this case you may need to increase the node's buffer.
  pub fn write_then<F>(&self, data: Bytes, callback: F)
  where
    F: Fn(GeneralResult<()>) + Send + Clone + Sync + 'static,
  {
    let targets = self.targets.clone();
    Self::inner_write(targets, data, self.timeout_ms, callback)
  }

  /// Override the default timeout.
  /// Write will be canceled if timeout, in this case you may need to increase the node's buffer.
  pub fn timed_write(&self, data: Bytes, timeout_ms: u64) {
    let targets = self.targets.clone();
    Self::inner_write(targets, data, Some(timeout_ms), |_| {})
  }

  /// Override the default timeout.
  /// Write will be canceled if timeout, in this case you may need to increase the node's buffer.
  pub fn timed_write_then<F>(&self, data: Bytes, timeout_ms: u64, callback: F)
  where
    F: Fn(GeneralResult<()>) + Send + Clone + Sync + 'static,
  {
    let targets = self.targets.clone();
    Self::inner_write(targets, data, Some(timeout_ms), callback)
  }

  fn inner_write<F>(
    targets: Arc<Mutex<Vec<Handle>>>,
    data: Bytes,
    timeout_ms: Option<u64>,
    callback: F,
  ) where
    F: Fn(GeneralResult<()>) + Send + Clone + Sync + 'static,
  {
    tokio::spawn(async move {
      for handle in targets.lock().await.iter() {
        if let Some(timeout_ms) = timeout_ms {
          handle.timed_write_then(data.clone(), timeout_ms.clone(), callback.clone());
        } else {
          handle.write_then(data.clone(), callback.clone());
        }
      }
    });
  }

  pub fn stop(self) {
    if self.propagate_stop {
      let targets = self.targets;
      tokio::spawn(async move {
        while let Some(handle) = targets.lock().await.pop() {
          handle.stop();
        }
      });
    }
  }

  pub fn stop_then<F>(self, callback: F)
  where
    F: Fn(GeneralResult<()>) + Send + Clone + Sync + 'static,
  {
    if self.propagate_stop {
      let targets = self.targets;
      tokio::spawn(async move {
        while let Some(handle) = targets.lock().await.pop() {
          handle.stop_then(callback.clone());
        }
      });
    }
  }
}
