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
  ws: TcpStream,
  tag: String,
}

impl WebsocketPeerBuilder {
  pub fn new(ws: TcpStream) -> Self {
    WebsocketPeerBuilder {
      ws,
      tag: String::from("websocket"),
    }
  }

  pub fn with_tag(&mut self, tag: String) -> &Self {
    self.tag = tag;
    self
  }

  pub async fn build(self, id: i32, hub_tx: Sender<HubEvent>, buffer: usize) -> Box<dyn Peer> {
    Box::new(WebsocketPeer::new(id, hub_tx, self.ws, buffer, self.tag).await)
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
