use std::net::SocketAddr;

use rua::model::{Result, StoppableHandle, Urx};
use tokio::sync::mpsc;
use tokio_kcp::KcpConfig;
use tokio_kcp::KcpListener as InnerKcpListener;

use crate::node::KcpNode;

pub struct KcpListener {
  addr: String,
  node_write_buffer: usize,
  handle: StoppableHandle,
  stop_rx: Urx,
  config: KcpConfig,
  peer_handler: Option<Box<dyn FnMut(KcpNode) + Send>>,
}

impl KcpListener {
  pub fn new(addr: String, node_write_buffer: usize, config: KcpConfig) -> Self {
    let (stop_tx, stop_rx) = mpsc::channel(1);
    Self {
      addr,
      stop_rx,
      config,
      handle: StoppableHandle::new(stop_tx),
      node_write_buffer,
      peer_handler: None,
    }
  }

  pub fn default_with_addr(addr: String) -> Self {
    Self::new(addr, 16, KcpConfig::default())
  }

  pub fn default() -> Self {
    Self::default_with_addr(String::from("127.0.0.1:7777"))
  }

  pub fn on_new_peer(mut self, f: impl FnMut(KcpNode) + 'static + Send) -> Self {
    self.peer_handler = Some(Box::new(f));
    self
  }

  /// Return `Err` if bind address failed.
  pub async fn spawn(self) -> Result<StoppableHandle> {
    let mut peer_handler = self
      .peer_handler
      .ok_or("missing peer_handler when spawn WsListener")?;
    let mut server = InnerKcpListener::bind(self.config, self.addr.parse::<SocketAddr>()?)?;
    let node_write_buffer = self.node_write_buffer;
    let mut stop_rx = self.stop_rx;

    // start ws listener
    tokio::spawn(async move {
      loop {
        tokio::select! {
          Ok((stream, _)) = server.accept() => {
            let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
            (peer_handler)(WsNode::new(ws_stream, node_write_buffer));
          },
          Some(()) = stop_rx.recv() => {
            break
          }
        }
      }
    });

    Ok(self.handle)
  }
}
