use std::net::SocketAddr;

use futures_util::StreamExt;
use quinn::{Endpoint, ServerConfig};
use rua::model::{GeneralResult, HandleBuilder, StopOnlyHandle, StopRx};
use tokio::sync::mpsc;

use crate::node::QuicNode;

pub struct QuicListener<'a> {
  addr: &'a str,
  config: ServerConfig,
  peer_write_buffer: usize,
  handle: StopOnlyHandle,
  stop_rx: StopRx,
  peer_handler: Option<Box<dyn FnMut(QuicNode) + Send>>,
}

impl<'a> QuicListener<'a> {
  pub fn bind(addr: &'a str) -> Self {
    let (stop_tx, stop_rx) = mpsc::channel(1);

    Self {
      addr,
      stop_rx,
      handle: HandleBuilder::default()
        .stop_tx(stop_tx)
        .build_stop_only()
        .unwrap(),
      peer_write_buffer: 16,
      peer_handler: None,
      config: ServerConfig::default(),
    }
  }

  pub fn config(mut self, config: ServerConfig) -> Self {
    self.config = config;
    self
  }

  pub fn peer_write_buffer(mut self, buffer: usize) -> Self {
    self.peer_write_buffer = buffer;
    self
  }

  pub fn on_new_peer(mut self, f: impl FnMut(QuicNode) + 'static + Send) -> Self {
    self.peer_handler = Some(Box::new(f));
    self
  }

  /// Return `Err` if bind address failed or mssing peer_handler.
  pub async fn spawn(self) -> GeneralResult<StopOnlyHandle> {
    let mut peer_handler = self
      .peer_handler
      .ok_or("missing peer_handler when spawn QuicListener")?;
    let mut eb = Endpoint::builder();
    eb.listen(self.config);
    let (_, mut incoming) = eb.bind(&self.addr.parse::<SocketAddr>()?)?;
    let peer_write_buffer = self.peer_write_buffer;
    let mut stop_rx = self.stop_rx;

    // start ws listener
    tokio::spawn(async move {
      loop {
        tokio::select! {
          conn = incoming.next() => {
            if let Some(conn) = conn {
              if let Ok(mut connection) = conn.await {
                // accept one stream
                if let Some(Ok((send, recv))) = connection.bi_streams.next().await {
                  (peer_handler)(QuicNode::new(send, recv, peer_write_buffer));
                } // else, client is down, ignore
              }
            } else {
              break // server is down
            }
          },
          Some(payload) = stop_rx.recv() => {
            (payload.callback)(Ok(()));
            break
          }
        }
      }
    });

    Ok(self.handle)
  }
}
