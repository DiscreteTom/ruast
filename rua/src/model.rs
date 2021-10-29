use bytes::Bytes;
use tokio::sync::mpsc;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub type Tx = mpsc::Sender<Bytes>;
pub type Rx = mpsc::Receiver<Bytes>;
pub type Utx = mpsc::Sender<()>;
pub type Urx = mpsc::Receiver<()>;

pub trait Writable {
  fn write(&self, data: Bytes);
}

pub trait Stoppable {
  fn stop(self);
}

#[derive(Clone)]
pub struct StopperHandle {
  stop_tx: Utx,
}

impl StopperHandle {
  pub fn new(stop_tx: Utx) -> Self {
    Self { stop_tx }
  }
}

impl Stoppable for StopperHandle {
  fn stop(self) {
    let stop_tx = self.stop_tx;
    tokio::spawn(async move { stop_tx.send(()).await });
  }
}

#[derive(Clone)]
pub struct WritableStopperHandle {
  tx: Tx,
  stop_tx: Utx,
}

impl WritableStopperHandle {
  pub fn new(tx: Tx, stop_tx: Utx) -> Self {
    Self { tx, stop_tx }
  }
}

impl Stoppable for WritableStopperHandle {
  fn stop(self) {
    let stop_tx = self.stop_tx.clone();
    tokio::spawn(async move { stop_tx.send(()).await });
  }
}

impl Writable for WritableStopperHandle {
  fn write(&self, data: Bytes) {
    let tx = self.tx.clone();
    tokio::spawn(async move { tx.send(data).await });
  }
}
