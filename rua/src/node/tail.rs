use std::{
  io::{ErrorKind, SeekFrom},
  time::Duration,
};

use bytes::{BufMut, Bytes, BytesMut};
use tokio::{
  io::{AsyncReadExt, AsyncSeekExt},
  sync::mpsc,
  time,
};

use crate::{
  go,
  model::{GeneralResult, HandleBuilder, StopOnlyHandle, StopRx},
  take, take_mut, take_option_mut,
};

pub struct TailNode<'a> {
  handle: StopOnlyHandle,
  filename: &'a str,
  stop_rx: StopRx,
  line_handler: Option<Box<dyn FnMut(Bytes) + Send>>,
  check_interval_ms: u64,
}

impl<'a> TailNode<'a> {
  pub fn with_file_name(filename: &'a str) -> Self {
    let (stop_tx, stop_rx) = mpsc::channel(1);

    Self {
      handle: HandleBuilder::default()
        .stop_tx(stop_tx)
        .build_stop_only()
        .unwrap(),
      filename,
      line_handler: None,
      stop_rx,
      check_interval_ms: 10,
    }
  }

  pub fn on_new_line<F>(mut self, f: F) -> Self
  where
    F: FnMut(Bytes) + Send + 'static,
  {
    self.line_handler = Some(Box::new(f));
    self
  }

  pub fn check_interval_ms(mut self, ms: u64) -> Self {
    self.check_interval_ms = ms;
    self
  }

  pub fn handle(&self) -> &StopOnlyHandle {
    &self.handle
  }

  pub async fn spawn(self) -> GeneralResult<StopOnlyHandle> {
    take_option_mut!(self, line_handler);

    let mut file = tokio::fs::OpenOptions::new()
      .read(true)
      .open(self.filename)
      .await?;

    // navigate to file end
    file.seek(SeekFrom::End(-1)).await?;

    take_mut!(self, stop_rx);
    take!(self, check_interval_ms);

    // reader thread
    go! {
      let mut buffer = BytesMut::with_capacity(64);

      loop {
        tokio::select! {
          Some(payload) = stop_rx.recv() => {
            (payload.callback)(Ok(()));
            break
          }
          b = file.read_u8() => {
            match b {
              Ok(b) => {
                if b == b'\n' {
                  // handle msg
                  (line_handler)(buffer.freeze());
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
              Err(err) => {
                if err.kind() == ErrorKind::UnexpectedEof {
                  // got file end, sleep for a while
                  time::sleep(Duration::from_millis(check_interval_ms)).await;
                } else {
                  break // other error
                }
              }
            }
          }
        }
      }
    };
    Ok(self.handle)
  }
}
