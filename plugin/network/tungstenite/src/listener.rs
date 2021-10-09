use rua::model::Event;
use std::net::TcpListener;
use std::sync::mpsc::Sender;
use tungstenite::accept;

use crate::peer::WebsocketPeerBuilder;

pub struct WebsocketListener {
  addr: String,
  op_code: u32,
  hub_tx: Sender<Event>,
  peer_tx: Sender<WebsocketPeerBuilder>,
}

impl WebsocketListener {
  pub fn new(
    addr: &str,
    op_code: u32,
    hub_tx: Sender<Event>,
    peer_tx: Sender<WebsocketPeerBuilder>,
  ) -> WebsocketListener {
    WebsocketListener {
      addr: String::from(addr),
      op_code,
      hub_tx,
      peer_tx,
    }
  }

  pub fn start(&self) {
    let server = TcpListener::bind(&self.addr).unwrap();
    while let Ok((stream, addr)) = server.accept(){
      tokio_tungstenite.
    }

    // for stream in server.incoming() {
    //   let websocket = accept(stream.unwrap()).unwrap();
    //   self.hub_tx.send(Event::Custom(self.op_code)).unwrap();
    //   self
    //     .peer_tx
    //     .send(WebsocketPeerBuilder::new(websocket))
    //     .unwrap();
    // }
  }
}
