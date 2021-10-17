use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use rua::{
  impl_peer, impl_peer_builder,
  model::{HubEvent, Peer, PeerBuilder, PeerMsg, Result},
};
use tokio::{net::TcpStream, sync::mpsc::Sender};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

#[derive(Debug)]
pub struct WebsocketPeerBuilder {
  tag: Option<String>,
  id: Option<u32>,
  hub_tx: Option<Sender<HubEvent>>,
  ws: Option<TcpStream>,
}

impl WebsocketPeerBuilder {
  pub fn new() -> Self {
    WebsocketPeerBuilder {
      tag: Some(String::from("websocket")),
      ws: None,
      id: None,
      hub_tx: None,
    }
  }

  pub fn ws(mut self, ws: TcpStream) -> Self {
    self.ws = Some(ws);
    self
  }

  pub fn boxed(self) -> Box<dyn PeerBuilder> {
    Box::new(self)
  }
}

#[async_trait]
impl PeerBuilder for WebsocketPeerBuilder {
  impl_peer_builder!(all);

  async fn build(&mut self) -> Result<Box<dyn Peer>> {
    let id = self.id.ok_or("id is required to build WebsocketPeer")?;
    let hub_tx = self
      .hub_tx
      .take()
      .ok_or("hub_tx is required to build WebsocketPeer")?;
    let ws = self
      .ws
      .take()
      .ok_or("ws is required to build WebsocketPeer")?;

    let ws_stream = tokio_tungstenite::accept_async(ws).await?;
    let (writer, mut reader) = ws_stream.split();

    // reader thread
    tokio::spawn(async move {
      loop {
        match reader.next().await {
          Some(msg) => {
            let msg = msg.expect("read websocket error");
            if msg.is_close() {
              // remove self from EventHub
              hub_tx.send(HubEvent::RemovePeer(id)).await.ok();
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

    Ok(Box::new(WebsocketPeer {
      id,
      tag: self.tag.take().unwrap(),
      writer,
    }))
  }
}

pub struct WebsocketPeer {
  id: u32,
  tag: String,
  writer: SplitSink<WebSocketStream<TcpStream>, Message>,
}

#[async_trait]
impl Peer for WebsocketPeer {
  impl_peer!(all);

  async fn write(&mut self, data: Bytes) -> Result<()> {
    Ok(self.writer.send(Message::Binary(data.to_vec())).await?)
  }
  fn stop(&mut self) {}
}
