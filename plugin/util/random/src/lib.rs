use bytes::Bytes;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rua::model::{GeneralResult, HandleBuilder, StopOnlyHandle, StopRx};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

pub struct RandomNode {
  handle: StopOnlyHandle,
  stop_rx: StopRx,
  msg_handler: Option<Box<dyn Fn(Bytes) + Send>>,
  nbyte: usize,
  interval_ms: u64,
}

impl Default for RandomNode {
  fn default() -> Self {
    let (stop_tx, stop_rx) = mpsc::channel(1);
    Self {
      stop_rx,
      msg_handler: None,
      handle: HandleBuilder::default()
        .stop_tx(stop_tx)
        .build_stop_only()
        .unwrap(),
      nbyte: 8,
      interval_ms: 200,
    }
  }
}

impl RandomNode {
  pub fn nbyte(mut self, n: usize) -> Self {
    self.nbyte = n;
    self
  }

  pub fn interval_ms(mut self, ms: u64) -> Self {
    self.interval_ms = ms;
    self
  }

  pub fn on_msg(mut self, f: impl Fn(Bytes) + 'static + Send) -> Self {
    self.msg_handler = Some(Box::new(f));
    self
  }

  pub fn handle(&self) -> &StopOnlyHandle {
    &self.handle
  }

  /// Return `Err` if missing `msg_handler`.
  pub fn spawn(self) -> GeneralResult<StopOnlyHandle> {
    let handler = self
      .msg_handler
      .ok_or("missing handler when create RandomNode")?;
    let interval_ms = self.interval_ms;
    let nbyte = self.nbyte;
    let mut stop_rx = self.stop_rx;

    tokio::spawn(async move {
      loop {
        tokio::select! {
          Some(payload) = stop_rx.recv() => {
            (payload.callback)(Ok(()));
            break
          }
          _ = time::sleep(Duration::from_millis(interval_ms)) => {
            (handler)(random_alphanumeric_bytes(nbyte))
          }
        }
      }
    });

    Ok(self.handle)
  }
}

fn random_alphanumeric_bytes(n: usize) -> Bytes {
  thread_rng().sample_iter(&Alphanumeric).take(n).collect()
}
