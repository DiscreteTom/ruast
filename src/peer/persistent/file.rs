use std::{
  error::Error,
  fs::{File, OpenOptions},
  io::{self, Write},
};

use crate::model::Peer;

pub struct FilePeer {
  tag: String,
  id: i32,
  file: File,
}

impl FilePeer {
  pub fn new(id: i32, filename: &str) -> Result<Box<dyn Peer>, io::Error> {
    Ok(Box::new(FilePeer {
      tag: String::from("file"),
      id,
      file: OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(filename)?,
    }))
  }
}

impl Peer for FilePeer {
  fn write(&mut self, data: std::sync::Arc<Vec<u8>>) -> Result<(), Box<dyn Error>> {
    self.file.write_all(&data)?;
    Ok(self.file.sync_data()?)
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
  fn start(&mut self) -> Result<(), Box<dyn Error>> {
    Ok(())
  }
}
