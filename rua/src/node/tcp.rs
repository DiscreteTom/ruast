use std::net::SocketAddr;

use bytes::{BufMut, Bytes, BytesMut};
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  net::{self, TcpStream},
  sync::mpsc,
};

use crate::{
  go,
  model::{
    GeneralResult, Handle, HandleBuilder, StopOnlyHandle, StopPayload, StopRx, StopTx, WriteRx,
  },
  take, take_mut, take_option_mut,
};

pub struct TcpListener<'a> {
  addr: &'a str,
  peer_handler: Option<Box<dyn FnMut(TcpNode) + Send>>,
  peer_write_buffer: usize,
  handle: StopOnlyHandle,
  stop_rx: StopRx,
}

impl<'a> TcpListener<'a> {
  pub fn bind(addr: &'a str) -> Self {
    let (stop_tx, stop_rx) = mpsc::channel(1);

    Self {
      addr,
      stop_rx,
      peer_handler: None,
      peer_write_buffer: 16,
      handle: HandleBuilder::default()
        .stop_tx(stop_tx)
        .build_stop_only()
        .unwrap(),
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

  pub fn handle(&self) -> &StopOnlyHandle {
    &self.handle
  }

  /// Return `Err` if missing `peer_handler` or failed bind to address.
  pub async fn spawn(self) -> GeneralResult<StopOnlyHandle> {
    take_option_mut!(self, peer_handler);

    let listener = net::TcpListener::bind(self.addr).await?;

    take!(self, peer_write_buffer);
    take_mut!(self, stop_rx);

    go! {
      loop {
        tokio::select! {
          result = listener.accept() => {
            if let Ok((socket, addr)) = result {
              peer_handler(TcpNode::new(socket, addr, peer_write_buffer));
            } else {
              break
            }
          }
          Some(payload) = stop_rx.recv() => {
            (payload.callback)(Ok(()));
            break
          }
        }
      }
    };

    Ok(self.handle)
  }
}

pub struct TcpNode {
  handle: Handle,
  socket: TcpStream,
  addr: SocketAddr,
  input_handler: Option<Box<dyn FnMut(Bytes) + Send>>,
  rx: WriteRx,
  stop_rx: StopRx,
  stop_tx: StopTx,
}

impl TcpNode {
  pub fn new(socket: TcpStream, addr: SocketAddr, buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(buffer);
    let (stop_tx, stop_rx) = mpsc::channel(1);

    Self {
      socket,
      addr,
      rx,
      stop_rx,
      input_handler: None,
      handle: HandleBuilder::default()
        .tx(tx)
        .stop_tx(stop_tx.clone())
        .build()
        .unwrap(),
      stop_tx,
    }
  }

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
    take_mut!(self, stop_rx, rx);
    take!(self, stop_tx);

    let (reader_stop_tx, mut reader_stop_rx) = mpsc::channel(1);
    let (writer_stop_tx, mut writer_stop_rx) = mpsc::channel(1);
    let (mut reader, mut writer) = self.socket.into_split();

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
    if let Some(mut input_handler) = self.input_handler {
      go! {
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
                Err(_) => break,
              }
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
              let result = async {
                writer.write_all(&payload.data).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
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
    };

    self.handle
  }
}
