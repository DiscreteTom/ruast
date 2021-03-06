use std::{
  error::Error,
  fmt::{self, Display},
  time::Duration,
};

use bytes::Bytes;
use tokio::{
  sync::mpsc::{Receiver, Sender},
  time,
};

use crate::{clone, go, take, take_option};

pub type GeneralResult<T> = std::result::Result<T, Box<dyn Error>>;

pub type CallbackFn = Box<dyn Fn(GeneralResult<()>) + Send + Sync>;
pub type WriteTx = Sender<WritePayload>;
pub type WriteRx = Receiver<WritePayload>;
pub type StopTx = Sender<StopPayload>;
pub type StopRx = Receiver<StopPayload>;

pub struct WritePayload {
  pub data: Bytes,
  pub callback: CallbackFn,
}

impl WritePayload {
  pub fn with_data(data: Bytes) -> Self {
    Self {
      data,
      callback: Box::new(|_| {}),
    }
  }

  pub fn callback<F>(mut self, callback: F) -> Self
  where
    F: Fn(GeneralResult<()>) + Send + Sync + 'static,
  {
    self.callback = Box::new(callback);
    self
  }
}

pub struct StopPayload {
  pub callback: CallbackFn,
}

impl StopPayload {
  pub fn with_callback<F>(callback: F) -> Self
  where
    F: Fn(GeneralResult<()>) + Send + Sync + 'static,
  {
    Self {
      callback: Box::new(callback),
    }
  }

  pub fn callback<F>(mut self, callback: F) -> Self
  where
    F: Fn(GeneralResult<()>) + Send + Sync + 'static,
  {
    self.callback = Box::new(callback);
    self
  }
}

impl Default for StopPayload {
  fn default() -> Self {
    Self {
      callback: Box::new(|_| {}),
    }
  }
}

#[derive(Debug, Default)]
pub struct HandleBuilder {
  tx: Option<WriteTx>,
  stop_tx: Option<StopTx>,
  timeout_ms: Option<u64>,
}

impl HandleBuilder {
  pub fn tx(mut self, tx: WriteTx) -> Self {
    self.tx = Some(tx);
    self
  }

  pub fn stop_tx(mut self, stop_tx: StopTx) -> Self {
    self.stop_tx = Some(stop_tx);
    self
  }

  pub fn timeout_ms(mut self, ms: u64) -> Self {
    self.timeout_ms = Some(ms);
    self
  }

  /// Return `Err` if missing `tx` or `stop_tx`.
  pub fn build(self) -> GeneralResult<Handle> {
    take!(self, timeout_ms);
    take_option!(self, tx, stop_tx);
    Ok(Handle {
      tx,
      stop_only: StopOnlyHandle { stop_tx },
      timeout_ms,
    })
  }

  /// Return `Err` if missing `stop_rx`.
  pub fn build_stop_only(self) -> GeneralResult<StopOnlyHandle> {
    take_option!(self, stop_tx);
    Ok(StopOnlyHandle { stop_tx })
  }
}

#[derive(Clone)]
pub struct StopOnlyHandle {
  stop_tx: StopTx,
}

impl StopOnlyHandle {
  pub fn stop(self) {
    take!(self, stop_tx);
    go! { stop_tx.send(StopPayload::default()).await.ok() }
  }

  pub fn stop_then<F>(self, callback: F)
  where
    F: Fn(GeneralResult<()>) + Send + Clone + Sync + 'static,
  {
    take!(self, stop_tx);
    go! {
      if stop_tx
        .send(StopPayload::with_callback(callback.clone()))
        .await
        .is_err()
      {
        callback(Err(Box::new(HandleError::ChannelClosed)));
      }
    };
  }
}

#[derive(Clone)]
pub struct Handle {
  tx: WriteTx,
  stop_only: StopOnlyHandle,
  timeout_ms: Option<u64>,
}

impl Handle {
  pub fn set_timeout_ms(&mut self, ms: u64) {
    self.timeout_ms = Some(ms);
  }

  pub fn clear_timeout(&mut self) {
    self.timeout_ms = None
  }

  /// Write will be canceled if timeout, in this case you may need to increase the node's buffer.
  pub fn write(&self, data: Bytes) {
    clone!(self, tx);
    Self::inner_write(tx, data, self.timeout_ms, |_| {})
  }

  /// Write will be canceled if timeout, in this case you may need to increase the node's buffer.
  pub fn write_then<F>(&self, data: Bytes, callback: F)
  where
    F: Fn(GeneralResult<()>) + Send + Clone + Sync + 'static,
  {
    clone!(self, tx);
    Self::inner_write(tx, data, self.timeout_ms, callback)
  }

  /// Override the default timeout.
  /// Write will be canceled if timeout, in this case you may need to increase the node's buffer.
  pub fn timed_write(&self, data: Bytes, timeout_ms: u64) {
    clone!(self, tx);
    Self::inner_write(tx, data, Some(timeout_ms), |_| {})
  }

  /// Override the default timeout.
  /// Write will be canceled if timeout, in this case you may need to increase the node's buffer.
  pub fn timed_write_then<F>(&self, data: Bytes, timeout_ms: u64, callback: F)
  where
    F: Fn(GeneralResult<()>) + Send + Clone + Sync + 'static,
  {
    let tx = self.tx.clone();
    Self::inner_write(tx, data, Some(timeout_ms), callback)
  }

  fn inner_write<F>(tx: WriteTx, data: Bytes, timeout_ms: Option<u64>, callback: F)
  where
    F: Fn(GeneralResult<()>) + Send + Clone + Sync + 'static,
  {
    go! {
      if let Some(timeout_ms) = timeout_ms {
        tokio::select! {
          result = tx.send(WritePayload::with_data(data).callback(callback.clone())) => {
            if result.is_err(){
              callback(Err(Box::new(HandleError::ChannelClosed)));
            }
          }
          _ = time::sleep(Duration::from_millis(timeout_ms)) => {
            callback(Err(Box::new(HandleError::Timeout)));
          }
        }
      } else {
        // no timeout
        let result = tx
          .send(WritePayload::with_data(data).callback(callback.clone()))
          .await;
        if result.is_err() {
          callback(Err(Box::new(HandleError::ChannelClosed)));
        }
      }
    };
  }

  pub fn stop(self) {
    self.stop_only.stop()
  }

  pub fn stop_then<F>(self, callback: F)
  where
    F: Fn(GeneralResult<()>) + Send + Clone + Sync + 'static,
  {
    self.stop_only.stop_then(callback)
  }

  pub fn stop_only_handle(&self) -> &StopOnlyHandle {
    &self.stop_only
  }
}

#[derive(Debug)]
pub enum HandleError {
  ChannelClosed,
  Timeout,
}

impl Display for HandleError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      HandleError::ChannelClosed => write!(f, "channel closed"),
      HandleError::Timeout => write!(f, "timeout"),
    }
  }
}

impl Error for HandleError {}
