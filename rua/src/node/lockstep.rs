use std::{sync::Arc, time::Duration};
use tokio::{
  sync::{
    mpsc::{self, Receiver, Sender},
    Mutex,
  },
  time,
};

/// Dynamic lockstep controller.
/// Use `set_step_length` to change the step length.
pub struct Dlc {
  step_length: Arc<Mutex<u64>>, // in ms
  stop_tx: Sender<()>,
}

impl Dlc {
  pub fn new(step_length_ms: u64) -> (Self, Receiver<u64>) {
    let (tx, rx) = mpsc::channel(1);

    (Self::with_tx(tx, step_length_ms), rx)
  }

  pub fn with_tx(tx: Sender<u64>, step_length_ms: u64) -> Self {
    let (stop_tx, mut stop_rx) = mpsc::channel(1);
    let step_length = Arc::new(Mutex::new(step_length_ms));

    {
      let step_length = step_length.clone();
      tokio::spawn(async move {
        let mut current = 0;
        loop {
          tokio::select! {
            _ = stop_rx.recv() => {
              break
            }
            _ = time::sleep(Duration::from_millis(*step_length.lock().await)) => {
              tx.send(current).await.expect("lockstep controller send failed");
              current += 1;
            }
          }
        }
      });
    }

    Self {
      step_length,
      stop_tx,
    }
  }

  pub async fn set_step_length(&mut self, step_length: u64) {
    *self.step_length.lock().await = step_length
  }

  pub async fn stop(self) {
    self.stop_tx.send(()).await.ok();
  }
}
