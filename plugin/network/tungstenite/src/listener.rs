use rua::model::Result;
use tokio::{
  net::TcpListener,
  sync::mpsc::{self, Receiver, Sender},
};

use crate::node::WsNode;

pub struct WsListener {
  addr: String,
  peer_tx: Sender<WsNode>,
  peer_rx: Receiver<WsNode>,
  node_buffer: usize,
}

impl WsListener {
  pub fn new(addr: &str, buffer: usize, node_buffer: usize) -> Self {
    let (peer_tx, peer_rx) = mpsc::channel(buffer);
    Self {
      addr: String::from(addr),
      peer_tx,
      peer_rx,
      node_buffer,
    }
  }

  /// Return Err if bind address failed.
  pub async fn spawn(self) -> Result<Receiver<WsNode>> {
    let server = TcpListener::bind(&self.addr).await?;
    let peer_tx = self.peer_tx;
    let node_buffer = self.node_buffer;

    tokio::spawn(async move {
      while let Ok((stream, _)) = server.accept().await {
        let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();

        peer_tx
          .send(WsNode::new(node_buffer).ws(ws_stream))
          .await
          .unwrap();
      }
    });

    Ok(self.peer_rx)
  }
}
