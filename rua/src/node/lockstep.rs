use std::time::Duration;

use tokio::{
  sync::mpsc,
  time::{self, Instant},
};

use crate::model::{Result, Urx, Utx};

pub struct Lockstep {
  step_handler: Option<Box<dyn Fn(u64) + Send>>,
  step_length_ms: u64,
  stop_rx: Urx,
  handle: LockstepHandle,
}

impl Lockstep {
  pub fn new() -> Self {
    let (stop_tx, stop_rx) = mpsc::channel(1);

    Self {
      stop_rx,
      step_handler: None,
      step_length_ms: 1000,
      handle: LockstepHandle::new(stop_tx),
    }
  }

  pub fn step_length_ms(mut self, ms: u64) -> Self {
    self.step_length_ms = ms;
    self
  }

  pub fn on_step(mut self, f: impl Fn(u64) + 'static + Send) -> Self {
    self.step_handler = Some(Box::new(f));
    self
  }

  pub fn spawn(self) -> Result<LockstepHandle> {
    let step_handler = self
      .step_handler
      .ok_or("missing step handler when build LsNode")?;
    let step_length_ms = self.step_length_ms;
    let mut stop_rx = self.stop_rx;

    tokio::spawn(async move {
      let mut current = 0;
      let mut timeout = Instant::now() + Duration::from_millis(step_length_ms);

      loop {
        tokio::select! {
          _ = time::sleep_until(timeout) => {
            (step_handler)(current);
            current += 1;
            timeout += Duration::from_millis(step_length_ms);
          }
          _ = stop_rx.recv() => {
            break
          }
        }
      }
    });
    Ok(self.handle)
  }
}

#[derive(Clone)]
pub struct LockstepHandle {
  stop_tx: Utx,
}

impl LockstepHandle {
  fn new(stop_tx: Utx) -> Self {
    Self { stop_tx }
  }
  pub fn stop(self) {
    let stop_tx = self.stop_tx;
    tokio::spawn(async move { stop_tx.send(()).await });
  }
}
