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

pub type GeneralResult<T> = std::result::Result<T, Box<dyn Error>>;

pub type CallbackFn = Box<dyn Fn(GeneralResult<()>) + Send + Sync>;
pub type WriteTx = Sender<WritePayload>;
pub type WriteRx = Receiver<WritePayload>;
pub type StopTx = Sender<StopPayload>;
pub type StopRx = Receiver<StopPayload>;

pub struct WritePayload {
  pub data: Bytes,
  pub callback: Option<CallbackFn>,
}

impl WritePayload {
  pub fn new(data: Bytes) -> Self {
    Self {
      data,
      callback: None,
    }
  }

  pub fn then<F>(mut self, callback: F) -> Self
  where
    F: Fn(GeneralResult<()>) + Send + Sync + 'static,
  {
    self.callback = Some(Box::new(callback));
    self
  }
}

#[derive(Default)]
pub struct StopPayload {
  pub callback: Option<CallbackFn>,
}

impl StopPayload {
  pub fn new<F>(callback: F) -> Self
  where
    F: Fn(GeneralResult<()>) + Send + Sync + 'static,
  {
    Self {
      callback: Some(Box::new(callback)),
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
    Ok(Handle {
      tx: self.tx.ok_or("missing tx when build Handle")?,
      stop_tx: self.stop_tx.ok_or("missing stop_tx when build handle")?,
      timeout_ms: self.timeout_ms,
    })
  }
}

#[derive(Clone)]
pub struct Handle {
  tx: WriteTx,
  stop_tx: StopTx,
  timeout_ms: Option<u64>,
}

impl Handle {
  pub fn set_timeout_ms(&mut self, ms: u64) {
    self.timeout_ms = Some(ms);
  }

  pub fn clear_timeout(&mut self) {
    self.timeout_ms = None
  }

  pub fn write(&self, data: Bytes) {
    if let Some(timeout_ms) = self.timeout_ms {
      self.timed_write(data, timeout_ms)
    } else {
      let tx = self.tx.clone();
      tokio::spawn(async move { tx.send(WritePayload::new(data)).await.ok() });
    }
  }

  pub fn write_then<F>(&self, data: Bytes, callback: F)
  where
    F: Fn(GeneralResult<()>) + Send + Clone + Sync + 'static,
  {
    if let Some(timeout_ms) = self.timeout_ms {
      self.timed_write_then(data, timeout_ms, callback)
    } else {
      let tx = self.tx.clone();
      tokio::spawn(async move {
        if tx
          .send(WritePayload::new(data).then(callback.clone()))
          .await
          .is_err()
        {
          callback(Err(Box::new(HandleError::ChannelClosed)));
        }
      });
    }
  }

  /// Override the default timeout.
  pub fn timed_write(&self, data: Bytes, timeout_ms: u64) {
    self.timed_write_then(data, timeout_ms, |_| {})
  }

  /// Override the default timeout.
  pub fn timed_write_then<F>(&self, data: Bytes, timeout_ms: u64, callback: F)
  where
    F: Fn(GeneralResult<()>) + Send + Clone + Sync + 'static,
  {
    let tx = self.tx.clone();
    tokio::spawn(async move {
      tokio::select! {
        result = tx.send(WritePayload::new(data).then(callback.clone())) => {
          if result.is_err(){
            callback(Err(Box::new(HandleError::ChannelClosed)));
          }
        }
        _ = time::sleep(Duration::from_millis(timeout_ms)) => {
          callback(Err(Box::new(HandleError::Timeout)));
        }
      }
    });
  }

  pub fn stop(self) {
    let stop_tx = self.stop_tx;
    tokio::spawn(async move { stop_tx.send(StopPayload::default()).await.ok() });
  }

  pub fn stop_then<F>(self, callback: F)
  where
    F: Fn(GeneralResult<()>) + Send + Clone + Sync + 'static,
  {
    let stop_tx = self.stop_tx;
    tokio::spawn(async move {
      if stop_tx
        .send(StopPayload::new(callback.clone()))
        .await
        .is_err()
      {
        callback(Err(Box::new(HandleError::ChannelClosed)));
      }
    });
  }
}

#[derive(Debug)]
pub enum HandleError {
  ChannelClosed,
  Timeout,
  NodeAlreadyStopped,
}

impl Display for HandleError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      HandleError::ChannelClosed => write!(f, "channel closed"),
      HandleError::Timeout => write!(f, "timeout"),
      HandleError::NodeAlreadyStopped => write!(f, "node already stopped"),
    }
  }
}

impl Error for HandleError {}
