use std::fmt;

use bytes::Bytes;
use rua_macro::{Stoppable, Writable};
use tokio::sync::mpsc;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub type Tx = mpsc::Sender<Bytes>;
pub type Rx = mpsc::Receiver<Bytes>;
pub type Utx = mpsc::Sender<()>;
pub type Urx = mpsc::Receiver<()>;

pub trait Writable {
  fn write(&self, data: Bytes) -> Result<()>;
}

pub trait Stoppable {
  fn stop(self);
}

#[derive(Clone, Stoppable)]
pub struct StopperHandle {
  stop_tx: Utx,
}

impl StopperHandle {
  pub fn new(stop_tx: Utx) -> Self {
    Self { stop_tx }
  }
}

#[derive(Clone, Writable, Stoppable)]
pub struct WritableStopperHandle {
  tx: Tx,
  stop_tx: Utx,
}

impl WritableStopperHandle {
  pub fn new(tx: Tx, stop_tx: Utx) -> Self {
    Self { tx, stop_tx }
  }
}

#[derive(Debug)]
pub enum Error {
  WriteToClosedChannel,
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::WriteToClosedChannel => write!(f, "write to closed channel"),
    }
  }
}

impl std::error::Error for Error {}
