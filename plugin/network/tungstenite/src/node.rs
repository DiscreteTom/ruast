use std::net::SocketAddr;

use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use rua::{
  go,
  model::{Handle, HandleBuilder, StopPayload, StopRx, StopTx, WriteRx},
  take, take_mut,
};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

pub struct WsNode {
  handle: Handle,
  ws: WebSocketStream<TcpStream>,
  addr: SocketAddr,
  rx: WriteRx,
  stop_rx: StopRx,
  stop_tx: StopTx,
  msg_handler: Option<Box<dyn FnMut(Bytes) + Send>>,
}

impl WsNode {
  pub fn new(ws: WebSocketStream<TcpStream>, buffer: usize, addr: SocketAddr) -> Self {
    let (tx, rx) = mpsc::channel(buffer);
    let (stop_tx, stop_rx) = mpsc::channel(1);

    Self {
      ws,
      rx,
      addr,
      stop_rx,
      msg_handler: None,
      handle: HandleBuilder::default()
        .tx(tx)
        .stop_tx(stop_tx.clone())
        .build()
        .unwrap(),
      stop_tx,
    }
  }

  pub fn on_msg(mut self, f: impl FnMut(Bytes) + 'static + Send) -> Self {
    self.msg_handler = Some(Box::new(f));
    self
  }

  pub fn handle(&self) -> &Handle {
    &self.handle
  }

  pub fn addr(&self) -> &SocketAddr {
    &self.addr
  }

  pub fn spawn(self) -> Handle {
    take_mut!(self, stop_rx, rx);
    take!(self, stop_tx);
    let (reader_stop_tx, mut reader_stop_rx) = mpsc::channel(1);
    let (writer_stop_tx, mut writer_stop_rx) = mpsc::channel(1);
    let (mut writer, mut reader) = self.ws.split();

    // stopper thread
    go! {
      if let Some(payload) = stop_rx.recv().await {
        reader_stop_tx.send(()).await.ok();
        writer_stop_tx.send(()).await.ok();
        (payload.callback)(Ok(()));
      }
      // else, all stop_tx are dropped, stop_rx is disabled

      // stop_rx is dropped, later stop_tx.send will throw ChannelClosed error.
    };

    // reader thread
    if let Some(mut msg_handler) = self.msg_handler {
      go! {
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
            Some(()) = reader_stop_rx.recv() => {
              break
            }
          }
        }
        // notify writer thread
        stop_tx.send(StopPayload::default()).await.ok();
      };
    }

    // writer thread
    go! {
      loop {
        tokio::select! {
          Some(()) = writer_stop_rx.recv() => {
            break
          }
          payload = rx.recv() => {
            if let Some(payload) = payload {
              let result =  writer.send(Message::Binary(payload.data.to_vec())).await;
              if let Err(e) = result {
                (payload.callback)(Err(Box::new(e)));
                break
              } else {
                (payload.callback)(Ok(()));
              }
            } else {
              break
            }
          }
        }
      }
    };

    self.handle
  }
}
