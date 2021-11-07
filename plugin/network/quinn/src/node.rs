use bytes::{BufMut, Bytes, BytesMut};
use quinn::{
  crypto::rustls::TlsSession,
  generic::{RecvStream, SendStream},
};
use rua::model::{Handle, HandleBuilder, StopPayload, StopRx, StopTx, WriteRx};
use tokio::{io::AsyncReadExt, sync::mpsc};

pub struct QuicNode {
  handle: Handle,
  sender: SendStream<TlsSession>,
  receiver: RecvStream<TlsSession>,
  rx: WriteRx,
  stop_rx: StopRx,
  stop_tx: StopTx,
  msg_handler: Option<Box<dyn FnMut(Bytes) + Send>>,
}

impl QuicNode {
  pub fn new(
    sender: SendStream<TlsSession>,
    receiver: RecvStream<TlsSession>,
    buffer: usize,
  ) -> Self {
    let (tx, rx) = mpsc::channel(buffer);
    let (stop_tx, stop_rx) = mpsc::channel(1);

    Self {
      sender,
      receiver,
      rx,
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

  pub fn spawn(self) -> Handle {
    let mut stop_rx = self.stop_rx;
    let stop_tx = self.stop_tx;
    let mut rx = self.rx;
    let (reader_stop_tx, mut reader_stop_rx) = mpsc::channel(1);
    let (writer_stop_tx, mut writer_stop_rx) = mpsc::channel(1);
    let mut reader = self.receiver;
    let mut writer = self.sender;

    // stopper thread
    tokio::spawn(async move {
      if let Some(payload) = stop_rx.recv().await {
        reader_stop_tx.send(()).await.ok();
        writer_stop_tx.send(()).await.ok();
        (payload.callback)(Ok(()));
      }
      // else, all stop_tx are dropped, stop_rx is disabled

      // stop_rx is dropped, later stop_tx.send will throw ChannelClosed error.
    });

    // reader thread
    if let Some(mut msg_handler) = self.msg_handler {
      tokio::spawn(async move {
        let mut buffer = BytesMut::with_capacity(64);

        loop {
          tokio::select! {
            Some(()) = reader_stop_rx.recv() => {
              break
            }
            b = reader.read_u8() => {
              match b {
                Ok(b) => {
                  if b == b'\n' {
                    // handle msg
                    (msg_handler)(buffer.freeze());
                    // reset buffer
                    buffer = BytesMut::with_capacity(64);
                  } else if b != b'\r' {
                    // append
                    if buffer.len() == buffer.capacity() {
                      buffer.reserve(64);
                    }
                    buffer.put_u8(b);
                  }
                }
                Err(_) => break,
              }
            }
          }
        }
        // notify writer thread
        stop_tx.send(StopPayload::default()).await.ok();
      });
    }

    // writer thread
    tokio::spawn(async move {
      loop {
        tokio::select! {
          Some(()) = writer_stop_rx.recv() => {
            break
          }
          payload = rx.recv() => {
            if let Some(payload) = payload {
              let result = async {
                writer.write_all(&payload.data).await?;
                writer.write_all(b"\n").await?;
                std::io::Result::Ok(())
              }
              .await;
              if let Err(e) = result {
                (payload.callback)(Err(Box::new(e)));
                break
              } else {
                (payload.callback)(Ok(()));
              }
            } else {
              break // all tx are dropped
            }
          }
        }
      }
      writer.finish().await.ok(); // gracefully shutdown
    });

    self.handle
  }
}
