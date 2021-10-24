use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use rua::{
  impl_node,
  model::{NodeEvent, Result, Rx, Tx},
};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

#[derive(Debug)]
pub struct WsNode {
  ws: Option<WebSocketStream<TcpStream>>,
  tx: Tx,
  rx: Rx,
  sink: Option<Tx>,
}

impl WsNode {
  impl_node!(tx, sink);

  pub fn new(buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(buffer);
    Self {
      ws: None,
      tx,
      rx,
      sink: None,
    }
  }

  pub fn ws(mut self, ws: WebSocketStream<TcpStream>) -> Self {
    self.ws = Some(ws);
    self
  }

  pub async fn spawn(self) -> Result<Tx> {
    let mut rx = self.rx;
    let (stop_tx, mut stop_rx) = mpsc::channel(1);
    let (mut writer, mut reader) = self
      .ws
      .ok_or("missing WebSocket stream when spawn WsNode")?
      .split();

    // reader thread
    if let Some(sink) = self.sink {
      tokio::spawn(async move {
        loop {
          tokio::select! {
            next = reader.next() => {
              match next {
                Some(msg) => {
                  let msg = msg.expect("read websocket error");
                  if msg.is_close() {
                    break;
                  } else {
                    sink
                      .send(NodeEvent::Write(Bytes::from(msg.into_data())))
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
    }

    // writer thread
    tokio::spawn(async move {
      loop {
        match rx.recv().await {
          None => break,
          Some(e) => match e {
            NodeEvent::Stop => break,
            NodeEvent::Write(data) => writer.send(Message::Binary(data.to_vec())).await.unwrap(),
          },
        }
      }
      stop_tx.send(()).await.ok();
    });

    Ok(self.tx)
  }
}
