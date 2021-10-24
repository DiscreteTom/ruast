use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use rua::{
  impl_node,
  model::{Brx, Btx, NodeEvent, ReaderNode, Rx, Tx, WriterNode},
  node::MockNode,
};
use rua_macro::{ReaderNode, WriterNode};
use tokio::{
  net::TcpStream,
  sync::{broadcast, mpsc},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

#[derive(ReaderNode, WriterNode, Debug)]
pub struct WsNode {
  ws: WebSocketStream<TcpStream>,
  tx: Tx,
  rx: Rx,
  btx: Btx,
}

impl WsNode {
  pub fn new(ws: WebSocketStream<TcpStream>, write_buffer: usize, read_buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(write_buffer);
    let (btx, _) = broadcast::channel(read_buffer);
    Self { ws, tx, rx, btx }
  }

  pub fn spawn(self) -> MockNode {
    let mut rx = self.rx;
    let btx = self.btx.clone();
    let (stop_tx, mut stop_rx) = mpsc::channel(1);
    let (mut writer, mut reader) = self.ws.split();

    // reader thread
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
                  if btx.send(NodeEvent::Write(Bytes::from(msg.into_data()))).is_err(){
                    break
                  }
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

    // writer thread
    tokio::spawn(async move {
      loop {
        match rx.recv().await {
          None => break,
          Some(e) => match e {
            NodeEvent::Stop => break,
            NodeEvent::Write(data) => {
              if writer.send(Message::Binary(data.to_vec())).await.is_err() {
                break;
              }
            }
          },
        }
      }
      stop_tx.send(()).await.ok();
    });

    MockNode::new(self.btx, self.tx)
  }
}
