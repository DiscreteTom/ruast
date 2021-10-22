use std::sync::Arc;

use tokio::sync::{
  mpsc::{self, Receiver, Sender},
  Mutex,
};

use crate::model::NodeEvent;

pub struct Broadcaster {
  targets: Arc<Mutex<Vec<Sender<NodeEvent>>>>,
  tx: Sender<NodeEvent>,
  stop_tx: Sender<()>,
}

impl Broadcaster {
  pub fn new(buffer: usize) -> Self {
    let (tx, mut rx): (Sender<NodeEvent>, Receiver<NodeEvent>) = mpsc::channel(buffer);
    let (stop_tx, mut stop_rx) = mpsc::channel(1);
    let targets: Arc<Mutex<Vec<Sender<NodeEvent>>>> = Arc::new(Mutex::new(Vec::new()));

    {
      let targets = targets.clone();
      tokio::spawn(async move {
        loop {
          tokio::select! {
            e = rx.recv() => {
              match e {
                Some(e)=>{
                  let mut targets = targets.lock().await;
                  let mut dead_peers: Vec<usize> = Vec::new();
                  for (i, p) in targets.iter().enumerate() {
                    if let Err(_) = p.send(e.clone()).await {
                      // this peer has been closed
                      dead_peers.push(i);
                    };
                  }
                  // remove dead peers
                  while let Some(dead) = dead_peers.pop() {
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
    }
  }

  pub fn tx(&self) -> &Sender<NodeEvent> {
    &self.tx
  }

  pub async fn add_target(&mut self, target: Sender<NodeEvent>) {
    self.targets.lock().await.push(target);
  }
}

impl Drop for Broadcaster {
  fn drop(&mut self) {
    let stop_tx = self.stop_tx.clone();
    tokio::spawn(async move {
      stop_tx.send(()).await.ok();
    });
  }
}
