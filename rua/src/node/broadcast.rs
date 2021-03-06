use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use tokio::sync::Mutex;

use crate::{
  clone, go,
  model::{GeneralResult, Handle},
};

#[derive(Clone, Default)]
pub struct Broadcaster {
  timeout_ms: Option<u64>,
  targets: Arc<Mutex<HashMap<usize, Handle>>>,
  keep_dead_targets: bool,
  current_handle_id: Arc<Mutex<usize>>,
}

impl Broadcaster {
  pub fn keep_dead_targets(mut self, enable: bool) -> Self {
    self.keep_dead_targets = enable;
    self
  }

  pub fn timeout_ms(mut self, ms: u64) -> Self {
    self.timeout_ms = Some(ms);
    self
  }

  pub fn add_target(&mut self, handle: Handle) {
    self.add_target_then(handle, |_| {})
  }

  pub fn add_target_then<F>(&mut self, handle: Handle, callback: F)
  where
    F: Fn(usize) + Send + Sync + 'static,
  {
    clone!(self, targets, current_handle_id);

    go! {
      let current_id;
      {
        let mut current_handle_id_locked = current_handle_id.lock().await;
        current_id = *current_handle_id_locked;
        *current_handle_id_locked += 1;
      }
      {
        targets.lock().await.insert(current_id, handle);
      }
      callback(current_id)
    };
  }

  pub fn remove_target(&self, id: usize) {
    self.remove_target_then(id, |_| {})
  }

  pub fn remove_target_then<F>(&self, id: usize, callback: F)
  where
    F: Fn(Option<Handle>) + Send + Sync + 'static,
  {
    clone!(self, targets);
    go! { callback(targets.lock().await.remove(&id)) };
  }

  /// Write will be canceled if timeout, in this case you may need to increase the node's buffer.
  pub fn write(&self, data: Bytes) {
    self.inner_write(data, self.timeout_ms, |_| {})
  }

  /// Write will be canceled if timeout, in this case you may need to increase the node's buffer.
  pub fn write_then<F>(&self, data: Bytes, callback: F)
  where
    F: Fn(GeneralResult<()>) + Send + Clone + Sync + 'static,
  {
    self.inner_write(data, self.timeout_ms, callback)
  }

  /// Override the default timeout.
  /// Write will be canceled if timeout, in this case you may need to increase the node's buffer.
  pub fn timed_write(&self, data: Bytes, timeout_ms: u64) {
    self.inner_write(data, Some(timeout_ms), |_| {})
  }

  /// Override the default timeout.
  /// Write will be canceled if timeout, in this case you may need to increase the node's buffer.
  pub fn timed_write_then<F>(&self, data: Bytes, timeout_ms: u64, callback: F)
  where
    F: Fn(GeneralResult<()>) + Send + Clone + Sync + 'static,
  {
    self.inner_write(data, Some(timeout_ms), callback)
  }

  fn inner_write<F>(&self, data: Bytes, timeout_ms: Option<u64>, callback: F)
  where
    F: Fn(GeneralResult<()>) + Send + Clone + Sync + 'static,
  {
    clone!(self, targets, keep_dead_targets);
    go! {
      let targets_locked = targets.lock().await;

      for (id, handle) in targets_locked.iter() {
        let targets = targets.clone();
        let keep_dead_targets = keep_dead_targets.clone();
        let id = *id;
        let callback = callback.clone();

        let callback = move |result: GeneralResult<()>| {
          let targets = targets.clone();
          if result.is_err() && !keep_dead_targets {
            go! { targets.lock().await.remove(&id) };
          }
          callback(result);
        };

        if let Some(timeout_ms) = timeout_ms {
          handle.timed_write_then(data.clone(), timeout_ms.clone(), callback.clone());
        } else {
          handle.write_then(data.clone(), callback.clone());
        }
      }
    };
  }

  pub fn stop_all(self) {
    self.stop_all_then(|_| {})
  }

  pub fn stop_all_then<F>(self, callback: F)
  where
    F: Fn(GeneralResult<()>) + Send + Clone + Sync + 'static,
  {
    clone!(self, targets);
    go! {
      let mut targets = targets.lock().await;
      let keys: Vec<usize> = targets.keys().map(|x| *x).collect();
      for id in keys {
        if let Some(handle) = targets.remove(&id) {
          handle.stop_then(callback.clone());
        }
      }
    };
  }
}
