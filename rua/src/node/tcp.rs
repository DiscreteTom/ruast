use std::net::SocketAddr;

use bytes::{BufMut, Bytes, BytesMut};
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  net::{self, TcpStream},
  sync::mpsc,
};

use crate::model::{GeneralResult, Handle, HandleBuilder, StopRx, WriteRx};

pub struct TcpListener<'a> {
  addr: &'a str,
  peer_handler: Option<Box<dyn FnMut(TcpNode) + Send>>,
  peer_write_buffer: usize,
}

impl<'a> TcpListener<'a> {
  pub fn bind(addr: &'a str) -> Self {
    Self {
      addr,
      peer_handler: None,
      peer_write_buffer: 16,
    }
  }

  pub fn peer_write_buffer(mut self, buffer: usize) -> Self {
    self.peer_write_buffer = buffer;
    self
  }

  pub fn on_new_peer(mut self, f: impl FnMut(TcpNode) + 'static + Send) -> Self {
    self.peer_handler = Some(Box::new(f));
    self
  }

  /// Return `Err` if missing `peer_handler` or failed bind to address.
  pub async fn spawn(self) -> GeneralResult<()> {
    let mut peer_handler = self
      .peer_handler
      .ok_or("missing peer_handler when spawn TcpListener")?;
    let listener = net::TcpListener::bind(self.addr).await?;
    let peer_write_buffer = self.peer_write_buffer;

    tokio::spawn(async move {
      while let Ok((socket, addr)) = listener.accept().await {
        let (tx, rx) = mpsc::channel(peer_write_buffer);
        let (stop_tx, stop_rx) = mpsc::channel(1);

        peer_handler(TcpNode {
          socket,
          addr,
          rx,
          stop_rx,
          input_handler: None,
          handle: HandleBuilder::default()
            .tx(tx)
            .stop_tx(stop_tx)
            .build()
            .unwrap(),
        })
      }
    });

    Ok(())
  }
}

pub struct TcpNode {
  handle: Handle,
  socket: TcpStream,
  addr: SocketAddr,
  input_handler: Option<Box<dyn FnMut(Bytes) + Send>>,
  rx: WriteRx,
  stop_rx: StopRx,
}

impl TcpNode {
  pub fn on_input<F>(mut self, f: F) -> Self
  where
    F: FnMut(Bytes) + Send + 'static,
  {
    self.input_handler = Some(Box::new(f));
    self
  }

  pub fn handle(&self) -> &Handle {
    &self.handle
  }

  pub fn addr(&self) -> &SocketAddr {
    &self.addr
  }

  pub fn spawn(self) -> Handle {
    let mut stop_rx = self.stop_rx;
    let mut rx = self.rx;
    let (reader_stop_tx, mut reader_stop_rx) = mpsc::channel(1);
    let (writer_stop_tx, mut writer_stop_rx) = mpsc::channel(1);
    let (mut reader, mut writer) = self.socket.into_split();

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
    if let Some(mut input_handler) = self.input_handler {
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
                    (input_handler)(buffer.freeze());
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
                Err(_) => break, // reader error
              }
            }
          }
        }
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
                writer.flush().await?;
                std::io::Result::Ok(())
              }
              .await;
              if let Err(e) = result {
                (payload.callback)(Err(Box::new(e)));
              } else {
                (payload.callback)(Ok(()));
              }
            } else {
              break // all tx are dropped
            }
          }
        }
      }
    });

    self.handle
  }
}
