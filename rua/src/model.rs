use bytes::Bytes;
use tokio::sync::{broadcast, mpsc};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
pub enum NodeEvent {
  Write(Bytes),
  Stop,
}

pub type Tx = mpsc::Sender<NodeEvent>;
pub type Rx = mpsc::Receiver<NodeEvent>;
pub type Btx = broadcast::Sender<NodeEvent>;
pub type Brx = broadcast::Receiver<NodeEvent>;

pub trait WriterNode {
  fn tx(&self) -> &Tx;
}

pub trait ReaderNode {
  fn brx(&self) -> Brx;
}
