use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use rua::model::{Rx, Urx, WritableStoppableHandle};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

pub struct WsNode {
  handle: WritableStoppableHandle,
  ws: WebSocketStream<TcpStream>,
  rx: Rx,
  stop_rx: Urx,
  msg_handler: Option<Box<dyn FnMut(Bytes) + Send>>,
}

impl WsNode {
  pub fn new(ws: WebSocketStream<TcpStream>, buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(buffer);
    let (stop_tx, stop_rx) = mpsc::channel(1);

    Self {
      ws,
      rx,
      stop_rx,
      msg_handler: None,
      handle: WritableStoppableHandle::new(tx, stop_tx),
    }
  }

  pub fn on_msg(mut self, f: impl FnMut(Bytes) + 'static + Send) -> Self {
    self.msg_handler = Some(Box::new(f));
    self
  }

  pub fn handle(&self) -> WritableStoppableHandle {
    self.handle.clone()
  }

  pub fn spawn(self) -> WritableStoppableHandle {
    let mut rx = self.rx;
    let mut stop_rx = self.stop_rx;
    let (mut writer, mut reader) = self.ws.split();

    // reader thread
    if let Some(mut msg_handler) = self.msg_handler {
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
                    msg_handler(Bytes::from(msg.into_data()));
                  }
                }
                None => break,
              }
            }
            Some(()) = stop_rx.recv() => {
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
          Some(data) => {
            if writer.send(Message::Binary(data.to_vec())).await.is_err() {
              break;
            }
          }
        }
      }
    });

    self.handle
  }
}
