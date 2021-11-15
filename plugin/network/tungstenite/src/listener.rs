use rua::{
  go,
  model::{GeneralResult, HandleBuilder, StopOnlyHandle, StopRx},
  take, take_mut, take_option_mut,
};
use tokio::{net::TcpListener, sync::mpsc};

use crate::node::WsNode;

pub struct WsListener<'a> {
  addr: &'a str,
  peer_write_buffer: usize,
  handle: StopOnlyHandle,
  stop_rx: StopRx,
  peer_handler: Option<Box<dyn FnMut(WsNode) + Send>>,
}

impl<'a> WsListener<'a> {
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
    }
  }

  pub fn peer_write_buffer(mut self, buffer: usize) -> Self {
    self.peer_write_buffer = buffer;
    self
  }

  pub fn on_new_peer(mut self, f: impl FnMut(WsNode) + 'static + Send) -> Self {
    self.peer_handler = Some(Box::new(f));
    self
  }

  /// Return `Err` if bind address failed or mssing peer_handler.
  pub async fn spawn(self) -> GeneralResult<StopOnlyHandle> {
    take_option_mut!(self, peer_handler);

    let server = TcpListener::bind(self.addr).await?;

    take!(self, peer_write_buffer);
    take_mut!(self, stop_rx);

    // start ws listener
    go! {
      loop {
        tokio::select! {
          Ok((stream, addr)) = server.accept() => {
            let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
            (peer_handler)(WsNode::new(ws_stream, peer_write_buffer, addr));
          },
          Some(payload) = stop_rx.recv() => {
            (payload.callback)(Ok(()));
            break
          }
        }
      }
    };

    Ok(self.handle)
  }
}
