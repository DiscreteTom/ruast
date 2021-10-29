use bytes::Bytes;
use tokio::sync::mpsc;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub type Tx = mpsc::Sender<Bytes>;
pub type Rx = mpsc::Receiver<Bytes>;
pub type Utx = mpsc::Sender<()>;
pub type Urx = mpsc::Receiver<()>;
