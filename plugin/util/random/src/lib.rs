use std::time::Duration;

use bytes::Bytes;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rua::{
  impl_node,
  model::{NodeEvent, Result, Rx, Tx},
};
use tokio::{sync::mpsc, time};

pub struct RandomNode {
  sink: Option<Tx>,
  tx: Tx,
  rx: Rx,
  nbyte: usize,
  interval: u64, // in ms
}

impl RandomNode {
  impl_node!(tx, sink);

  pub fn new(buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(buffer);
    Self {
      tx,
      rx,
      sink: None,
      nbyte: 8,
      interval: 200,
    }
  }

  pub fn nbyte(mut self, n: usize) -> Self {
    self.nbyte = n;
    self
  }

  pub fn interval(mut self, ms: u64) -> Self {
    self.interval = ms;
    self
  }

  pub fn spawn(self) -> Result<Tx> {
    let sink = self.sink.ok_or("missing sink when build RandomNode")?;
    let interval = self.interval;
    let nbyte = self.nbyte;
    let mut rx = self.rx;

    tokio::spawn(async move {
      loop {
        tokio::select! {
          e = rx.recv() => {
            match e {
              None => break,
              Some(e) => {
                if let NodeEvent::Stop = e {
                  break
                }
              }
            };
          }
          _ = time::sleep(Duration::from_millis(interval)) => {
            sink.send(NodeEvent::Write(random_alphanumeric_bytes(nbyte)))
              .await.expect("RandomNode failed to write to sink");
          }
        }
      }
    });

    Ok(self.tx)
  }
}

fn random_alphanumeric_bytes(n: usize) -> Bytes {
  thread_rng().sample_iter(&Alphanumeric).take(n).collect()
}
