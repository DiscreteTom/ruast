use std::sync::Arc;

use tokio::sync::{
  mpsc::{self, Sender},
  Mutex,
};

use crate::model::{NodeEvent, Rx, Tx};

/// Broadcaster.
/// You can add targets to the broadcaster dynamically by calling `add_target`.
/// Broadcaster will automatically remove dead targets from itself.
pub struct Bc {
  targets: Arc<Mutex<Vec<Tx>>>,
  tx: Tx,
  stop_tx: Sender<()>,
  stop_targets_on_drop: bool,
}

impl Bc {
  pub fn new(buffer: usize) -> Self {
    let (tx, mut rx): (Tx, Rx) = mpsc::channel(buffer);
    let (stop_tx, mut stop_rx) = mpsc::channel(1);
    let targets: Arc<Mutex<Vec<Tx>>> = Arc::new(Mutex::new(Vec::new()));

    {
      let targets = targets.clone();
      tokio::spawn(async move {
        loop {
          tokio::select! {
            e = rx.recv() => {
              match e {
                Some(e)=>{
                  let mut targets = targets.lock().await;
                  let mut dead_targets: Vec<usize> = Vec::new();
                  for (i, p) in targets.iter().enumerate() {
                    if let Err(_) = p.send(e.clone()).await {
                      // this target has been closed
                      dead_targets.push(i);
                    };
                  }
                  // remove dead targets
                  while let Some(dead) = dead_targets.pop() {
                    targets.remove(dead);
                  }
                }
                None=>break,
              }
            }
            _ = stop_rx.recv() => {
              break
            }
          };
        }
      });
    }

    Self {
      tx,
      stop_tx,
      targets,
      stop_targets_on_drop: false,
    }
  }

  pub fn stop_targets_on_drop(&mut self, enable: bool) {
    self.stop_targets_on_drop = enable;
  }

  pub fn tx(&self) -> &Tx {
    &self.tx
  }

  pub async fn add_target(&mut self, target: Tx) {
    self.targets.lock().await.push(target);
  }
}

impl Drop for Bc {
  fn drop(&mut self) {
    let stop_tx = self.stop_tx.clone();
    tokio::spawn(async move {
      stop_tx.send(()).await.ok(); // stop self
    });

    if self.stop_targets_on_drop {
      let tx = self.tx.clone();
      tokio::spawn(async move {
        tx.send(NodeEvent::Stop).await.ok(); // stop all targets
      });
    }
  }
}
