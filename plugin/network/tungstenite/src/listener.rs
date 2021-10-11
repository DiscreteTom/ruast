use rua::model::HubEvent;
use tokio::{
  net::TcpListener,
  sync::mpsc::{self, Receiver, Sender},
};

use crate::peer::WebsocketPeerBuilder;

pub struct WebsocketListener {
  addr: String,
  op_code: u32,
  hub_tx: Sender<HubEvent>,
  peer_tx: Sender<WebsocketPeerBuilder>,
}

impl WebsocketListener {
  pub fn new(
    addr: &str,
    op_code: u32,
    hub_tx: Sender<HubEvent>,
    buffer: usize,
  ) -> (Self, Receiver<WebsocketPeerBuilder>) {
    let (peer_tx, peer_rx) = mpsc::channel(buffer);
    (
      WebsocketListener {
        addr: String::from(addr),
        op_code,
        hub_tx,
        peer_tx,
      },
      peer_rx,
    )
  }

  pub async fn start(&self) {
    let server = TcpListener::bind(&self.addr).await.unwrap();
    while let Ok((stream, _)) = server.accept().await {
      self
        .hub_tx
        .send(HubEvent::Custom(self.op_code))
        .await
        .unwrap();
      self
        .peer_tx
        .send(WebsocketPeerBuilder::new().ws(stream))
        .await
        .unwrap();
    }
  }
}
