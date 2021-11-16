use std::time::Duration;

use tokio::{sync::mpsc, time};

use crate::{
  go,
  model::{GeneralResult, HandleBuilder, StopOnlyHandle, StopRx},
  take, take_mut, take_option_mut,
};

pub struct Ticker {
  tick_handler: Option<Box<dyn FnMut(u64) + Send>>,
  interval_ms: u64,
  stop_rx: StopRx,
  handle: StopOnlyHandle,
}

impl Default for Ticker {
  fn default() -> Self {
    Self::with_interval(1000)
  }
}

impl Ticker {
  pub fn with_interval(ms: u64) -> Self {
    let (stop_tx, stop_rx) = mpsc::channel(1);

    Self {
      stop_rx,
      tick_handler: None,
      interval_ms: ms,
      handle: HandleBuilder::default()
        .stop_tx(stop_tx)
        .build_stop_only()
        .unwrap(),
    }
  }

  pub fn handle(&self) -> &StopOnlyHandle {
    &self.handle
  }

  pub fn interval_ms(mut self, ms: u64) -> Self {
    self.interval_ms = ms;
    self
  }

  pub fn on_tick(mut self, f: impl FnMut(u64) + 'static + Send) -> Self {
    self.tick_handler = Some(Box::new(f));
    self
  }

  /// Return `Err` if missing `tick_handler`.
  pub fn spawn(self) -> GeneralResult<StopOnlyHandle> {
    take_option_mut!(self, tick_handler);
    take!(self, interval_ms);
    take_mut!(self, stop_rx);

    go! {
      let mut current = 0;
      let mut timer = time::interval(Duration::from_millis(interval_ms));

      loop {
        tokio::select! {
          _ = timer.tick() => {
            (tick_handler)(current);
            current += 1;
          }
          Some(payload) = stop_rx.recv() => {
            (payload.callback)(Ok(()));
            break
          }
        }
      }
    };
    Ok(self.handle)
  }
}
