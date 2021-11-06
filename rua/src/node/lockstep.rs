use std::time::Duration;

use tokio::{
  sync::mpsc,
  time::{self, Instant},
};

use crate::model::{GeneralResult, HandleBuilder, StopOnlyHandle, StopRx};

pub struct Lockstep {
  step_handler: Option<Box<dyn FnMut(u64) + Send>>,
  step_length_ms: u64,
  stop_rx: StopRx,
  handle: StopOnlyHandle,
}

impl Default for Lockstep {
  fn default() -> Self {
    Self::with_step_length_ms(1000)
  }
}

impl Lockstep {
  pub fn with_step_length_ms(ms: u64) -> Self {
    let (stop_tx, stop_rx) = mpsc::channel(1);

    Self {
      stop_rx,
      step_handler: None,
      step_length_ms: ms,
      handle: HandleBuilder::default()
        .stop_tx(stop_tx)
        .build_stop_only()
        .unwrap(),
    }
  }

  pub fn handle(&self) -> &StopOnlyHandle {
    &self.handle
  }

  pub fn step_length_ms(mut self, ms: u64) -> Self {
    self.step_length_ms = ms;
    self
  }

  pub fn on_step(mut self, f: impl FnMut(u64) + 'static + Send) -> Self {
    self.step_handler = Some(Box::new(f));
    self
  }

  pub fn spawn(self) -> GeneralResult<StopOnlyHandle> {
    let mut step_handler = self
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
          Some(payload) = stop_rx.recv() => {
            (payload.callback)(Ok(()));
            break
          }
        }
      }
    });
    Ok(self.handle)
  }
}
