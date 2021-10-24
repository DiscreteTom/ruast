use rua::model::Result;
use tokio::net::TcpListener;

use crate::node::WsNode;

pub struct WsListener {
  addr: String,
  node_write_buffer: usize,
  node_read_buffer: usize,
  peer_handler: Option<Box<dyn Fn(WsNode) + 'static + Send>>,
}

impl WsListener {
  pub fn new(addr: String, node_write_buffer: usize, node_read_buffer: usize) -> Self {
    Self {
      addr,
      node_write_buffer,
      node_read_buffer,
      peer_handler: None,
    }
  }

  pub fn default_with_addr(addr: String) -> Self {
    Self::new(addr, 16, 16)
  }

  pub fn default() -> Self {
    Self::default_with_addr(String::from("127.0.0.1:8080"))
  }

  pub fn peer_handler(mut self, f: impl Fn(WsNode) + 'static + Send) -> Self {
    self.peer_handler = Some(Box::new(f));
    self
  }

  /// Return `Err` if bind address failed.
  pub async fn spawn(self) -> Result<()> {
    let peer_handler = self
      .peer_handler
      .ok_or("missing peer_handler when spawn WsListener")?;
    let server = TcpListener::bind(&self.addr).await?;
    let node_write_buffer = self.node_write_buffer;
    let node_read_buffer = self.node_read_buffer;

    // start ws listener
    tokio::spawn(async move {
      while let Ok((stream, _)) = server.accept().await {
        let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
        (peer_handler)(WsNode::new(ws_stream, node_write_buffer, node_read_buffer));
      }
    });

    Ok(())
  }
}
