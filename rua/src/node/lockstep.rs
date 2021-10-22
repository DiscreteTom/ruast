use std::{sync::Arc, time::Duration};
use tokio::{
  sync::{
    mpsc::{self, Receiver, Sender},
    Mutex,
  },
  time,
};

pub struct LockstepController {
  step_length: Arc<Mutex<u64>>, // in ms
  stop_tx: Sender<()>,
}

impl LockstepController {
  pub fn new(step_length_ms: u64) -> (Self, Receiver<u64>) {
    let (stop_tx, mut stop_rx) = mpsc::channel(1);
    let (tx, rx) = mpsc::channel(1);
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

    (
      Self {
        step_length,
        stop_tx,
      },
      rx,
    )
  }

  pub async fn set_step_length(&mut self, step_length: u64) {
    *self.step_length.lock().await = step_length
  }

  pub async fn stop(self) {
    self.stop_tx.send(()).await.ok();
  }
}
