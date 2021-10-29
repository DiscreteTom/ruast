use bytes::Bytes;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rua::model::{Result, Urx, Utx};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

pub struct RandomNode {
  handle: RandomNodeHandle,
  stop_rx: Urx,
  msg_handler: Option<Box<dyn Fn(Bytes) + Send>>,
  nbyte: usize,
  interval_ms: u64,
}

impl RandomNode {
  pub fn new() -> Self {
    let (stop_tx, stop_rx) = mpsc::channel(1);
    Self {
      stop_rx,
      msg_handler: None,
      handle: RandomNodeHandle::new(stop_tx),
      nbyte: 8,
      interval_ms: 200,
    }
  }

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

  pub fn spawn(self) -> Result<RandomNodeHandle> {
    let handler = self
      .msg_handler
      .ok_or("missing handler when create RandomNode")?;
    let interval_ms = self.interval_ms;
    let nbyte = self.nbyte;
    let mut stop_rx = self.stop_rx;

    tokio::spawn(async move {
      loop {
        tokio::select! {
          _ = stop_rx.recv() => {
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

pub struct RandomNodeHandle {
  stop_tx: Utx,
}

impl RandomNodeHandle {
  fn new(stop_tx: Utx) -> Self {
    Self { stop_tx }
  }

  pub fn stop(self) {
    let stop_tx = self.stop_tx.clone();
    tokio::spawn(async move { stop_tx.send(()).await });
  }
}

fn random_alphanumeric_bytes(n: usize) -> Bytes {
  thread_rng().sample_iter(&Alphanumeric).take(n).collect()
}
