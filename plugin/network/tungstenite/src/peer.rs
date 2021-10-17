use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use rua::{
  impl_peer, impl_peer_builder,
  model::{Peer, PeerBuilder, PeerMsg, Result, ServerEvent},
};
use tokio::{
  net::TcpStream,
  sync::mpsc::{self, Sender},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

#[derive(Debug)]
pub struct WebsocketPeerBuilder {
  tag: Option<String>,
  id: Option<u32>,
  server_tx: Option<Sender<ServerEvent>>,
  ws: Option<TcpStream>,
}

impl WebsocketPeerBuilder {
  pub fn new() -> Self {
    WebsocketPeerBuilder {
      tag: Some(String::from("websocket")),
      ws: None,
      id: None,
      server_tx: None,
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

  async fn build(&mut self) -> Result<Box<dyn Peer + Send>> {
    let id = self.id.ok_or("id is required to build WebsocketPeer")?;
    let server_tx = self
      .server_tx
      .take()
      .ok_or("server_tx is required to build WebsocketPeer")?;
    let ws = self
      .ws
      .take()
      .ok_or("ws is required to build WebsocketPeer")?;

    let ws_stream = tokio_tungstenite::accept_async(ws).await?;
    let (writer, mut reader) = ws_stream.split();
    let (stop_tx, mut stop_rx) = mpsc::channel(1);

    // reader thread
    tokio::spawn(async move {
      loop {
        tokio::select! {
          next = reader.next() => {
            match next {
              Some(msg) => {
                let msg = msg.expect("read websocket error");
                if msg.is_close() {
                  // remove self from PeerManager
                  server_tx.send(ServerEvent::RemovePeer(id)).await.ok();
                  break;
                } else {
                  server_tx
                    .send(ServerEvent::PeerMsg(PeerMsg {
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
          _ = stop_rx.recv() => {
            break
          }
        }
      }
    });

    Ok(Box::new(WebsocketPeer {
      id,
      tag: self.tag.take().unwrap(),
      writer,
      stop_tx: Some(stop_tx),
    }))
  }
}

pub struct WebsocketPeer {
  id: u32,
  tag: String,
  writer: SplitSink<WebSocketStream<TcpStream>, Message>,
  stop_tx: Option<Sender<()>>,
}

#[async_trait]
impl Peer for WebsocketPeer {
  impl_peer!(all);

  async fn write(&mut self, data: Bytes) -> Result<()> {
    Ok(self.writer.send(Message::Binary(data.to_vec())).await?)
  }

  fn stop(&mut self) {
    let tx = self
      .stop_tx
      .take()
      .expect("WebsocketPeer can not be stopped twice");
    tokio::spawn(async move { tx.send(()).await });
  }
}
