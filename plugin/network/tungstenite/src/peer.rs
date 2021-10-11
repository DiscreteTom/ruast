use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use rua::model::{HubEvent, Peer, PeerEvent, PeerMsg};
use rua_macro::BasicPeer;
use tokio::{
  net::TcpStream,
  sync::mpsc::{self, Sender},
};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug)]
pub struct WebsocketPeerBuilder {
  tag: String,
  ws: Option<TcpStream>,
  id: Option<i32>,
  hub_tx: Option<Sender<HubEvent>>,
  buffer: Option<usize>,
}

impl WebsocketPeerBuilder {
  pub fn new() -> Self {
    WebsocketPeerBuilder {
      tag: String::from("websocket"),
      ws: None,
      id: None,
      hub_tx: None,
      buffer: None,
    }
  }

  pub fn ws(mut self, ws: TcpStream) -> Self {
    self.ws = Some(ws);
    self
  }
  pub fn id(mut self, id: i32) -> Self {
    self.id = Some(id);
    self
  }
  pub fn hub_tx(mut self, hub_tx: Sender<HubEvent>) -> Self {
    self.hub_tx = Some(hub_tx);
    self
  }
  pub fn buffer(mut self, buffer: usize) -> Self {
    self.buffer = Some(buffer);
    self
  }

  pub fn tag(mut self, tag: String) -> Self {
    self.tag = tag;
    self
  }

  pub async fn build(self) -> Box<dyn Peer> {
    Box::new(
      WebsocketPeer::new(
        self.id.unwrap(),
        self.hub_tx.unwrap(),
        self.ws.unwrap(),
        self.buffer.unwrap(),
        self.tag,
      )
      .await,
    )
  }
}

#[derive(BasicPeer)]
pub struct WebsocketPeer {
  id: i32,
  tag: String,
  tx: Sender<PeerEvent>,
}

impl WebsocketPeer {
  async fn new(
    id: i32,
    hub_tx: Sender<HubEvent>,
    ws: TcpStream,
    buffer: usize,
    tag: String,
  ) -> Self {
    let (tx, mut rx) = mpsc::channel(buffer);

    let ws_stream = tokio_tungstenite::accept_async(ws).await.unwrap();
    let (mut writer, mut reader) = ws_stream.split();

    // reader thread
    tokio::spawn(async move {
      loop {
        match reader.next().await {
          Some(msg) => {
            let msg = msg.unwrap();
            if msg.is_close() {
              // remove self from EventHub
              hub_tx.send(HubEvent::RemovePeer(id)).await.unwrap();
              break;
            } else {
              hub_tx
                .send(HubEvent::PeerMsg(PeerMsg {
                  peer_id: id,
                  data: Bytes::from(msg.into_data()),
                }))
                .await
                .unwrap();
            }
          }
          None => break,
        }
      }
    });

    // writer thread
    tokio::spawn(async move {
      loop {
        match rx.recv().await.unwrap() {
          PeerEvent::Write(data) => {
            writer.send(Message::Binary(data.to_vec())).await.unwrap();
          }
          PeerEvent::Stop => break,
        }
      }
    });

    Self { id, tx, tag }
  }
}
