use crate::model::{PeerBuilder, PeerIdAllocator};

pub struct SimpleIdGenerator {
  current: u32,
  start: u32,
}

impl SimpleIdGenerator {
  pub fn new(start: u32) -> Self {
    Self {
      start,
      current: start,
    }
  }
  pub fn next(&mut self) -> u32 {
    self.current += 1;
    if self.current == self.start {
      panic!("id overflow")
    }
    self.current
  }
}

pub struct SimplePeerIdAllocator {
  inner: SimpleIdGenerator,
}

impl SimplePeerIdAllocator {
  pub fn new(start: u32) -> Self {
    Self {
      inner: SimpleIdGenerator::new(start),
    }
  }
}

impl PeerIdAllocator for SimplePeerIdAllocator {
  fn allocate(&mut self, _: &Box<dyn PeerBuilder>) -> u32 {
    self.inner.next()
  }
}
