use std::{
  net::TcpStream,
  sync::{mpsc::Sender, Arc, Mutex},
  thread,
  time::SystemTime,
};

use rua::model::{Data, Event, Peer, PeerMsg, Result};
use tungstenite::{Message, WebSocket};

pub struct WebsocketPeerBuilder {
  ws: WebSocket<TcpStream>,
}

impl WebsocketPeerBuilder {
  pub fn new(ws: WebSocket<TcpStream>) -> Self {
    WebsocketPeerBuilder { ws }
  }

  pub fn build(self, id: i32, hub_tx: Sender<Event>) -> Box<dyn Peer> {
    Box::new(WebsocketPeer::new(id, hub_tx, self.ws))
  }
}

pub struct WebsocketPeer {
  id: i32,
  tag: String,
  hub_tx: Sender<Event>,
  ws: Arc<WebSocket<TcpStream>>,
}

impl WebsocketPeer {
  pub fn new(id: i32, hub_tx: Sender<Event>, ws: WebSocket<TcpStream>) -> WebsocketPeer {
    WebsocketPeer {
      id,
      hub_tx,
      ws: Arc::new(ws),
      tag: String::from("websocket"),
    }
  }
}

impl Peer for WebsocketPeer {
  fn write(&mut self, data: Data) -> Result<()> {
    Ok(self.ws.write_message(Message::Binary(data.to_vec()))?)
  }

  fn id(&self) -> i32 {
    self.id
  }

  fn set_tag(&mut self, tag: &str) {
    self.tag = String::from(tag);
  }

  fn tag(&self) -> &str {
    &self.tag
  }

  fn start(&mut self) -> Result<()> {
    let ws = self.ws.clone();
    let hub_tx = self.hub_tx.clone();
    let peer_id = self.id;
    thread::spawn(move || loop {
      let msg = ws.read_message().unwrap();
      hub_tx
        .send(Event::PeerMsg(PeerMsg {
          peer_id,
          data: Arc::new(msg.into_data()),
          time: SystemTime::now(),
        }))
        .unwrap();
    });

    Ok(())
  }
}
