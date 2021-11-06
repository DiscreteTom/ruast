use std::io;

use tokio::sync::mpsc;

use crate::model::{HandleBuilder, StopOnlyHandle, StopRx};

pub struct Ctrlc {
  handle: StopOnlyHandle,
  stop_rx: StopRx,
  signal_handler: Box<dyn FnOnce() + Send>,
}

impl Default for Ctrlc {
  fn default() -> Self {
    let (stop_tx, stop_rx) = mpsc::channel(1);
    Self {
      handle: HandleBuilder::default()
        .stop_tx(stop_tx)
        .build_stop_only()
        .unwrap(),
      stop_rx,
      signal_handler: Box::new(|| {}),
    }
  }
}

impl Ctrlc {
  pub fn on_signal<F>(mut self, f: F) -> Self
  where
    F: FnOnce() + Send + 'static,
  {
    self.signal_handler = Box::new(f);
    self
  }

  pub fn handle(&self) -> &StopOnlyHandle {
    &self.handle
  }

  pub fn spawn(self) -> StopOnlyHandle {
    let stop_rx = self.stop_rx;
    let signal_handler = self.signal_handler;
    tokio::spawn(async move { Self::inner_wait(stop_rx, signal_handler).await });
    self.handle
  }

  pub async fn wait(self) -> io::Result<()> {
    Self::inner_wait(self.stop_rx, self.signal_handler).await
  }

  async fn inner_wait(
    mut stop_rx: StopRx,
    signal_handler: Box<dyn FnOnce() + Send>,
  ) -> io::Result<()> {
    tokio::select! {
      Some(payload) = stop_rx.recv() => {
        (payload.callback)(Ok(()));
        Ok(())
      }
      result = tokio::signal::ctrl_c() => {
        match result {
          Ok(()) => {
            (signal_handler)();
            Ok(())
          }
          Err(e) => {
            Err(e)
          }
        }
      }
    }
  }
}
