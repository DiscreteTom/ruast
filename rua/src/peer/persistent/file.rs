use std::{
  cell::RefCell,
  fs::{File, OpenOptions},
  io::Write,
};

use bytes::Bytes;

use crate::model::{Peer, Result};

pub struct FilePeer {
  tag: String,
  id: i32,
  file: RefCell<File>,
}

impl FilePeer {
  pub fn new(id: i32, filename: &str) -> Result<Box<dyn Peer>> {
    Ok(Box::new(FilePeer {
      tag: String::from("file"),
      id,
      file: RefCell::new(
        OpenOptions::new()
          .create(true)
          .write(true)
          .append(true)
          .open(filename)?,
      ),
    }))
  }
}

impl Peer for FilePeer {
  fn write(&self, data: Bytes) -> Result<()> {
    self.file.borrow_mut().write_all(&data)?;
    Ok(self.file.borrow_mut().sync_data()?)
  }
  fn id(&self) -> i32 {
    self.id
  }
  fn set_tag(&mut self, tag: &str) {
    self.tag = String::from(tag);
  }
  fn tag(&self) -> &str {
    &self.tag
  }
}
