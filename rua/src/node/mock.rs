use crate::model::{Brx, Btx, ReaderNode, Tx, WriterNode};
use rua_macro::{ReaderNode, WriterNode};

#[derive(ReaderNode)]
pub struct MockReaderNode {
  btx: Btx,
}

impl MockReaderNode {
  pub fn new(btx: Btx) -> Self {
    Self { btx }
  }
}

#[derive(WriterNode)]
pub struct MockWriterNode {
  tx: Tx,
}

impl MockWriterNode {
  pub fn new(tx: Tx) -> Self {
    Self { tx }
  }

  pub fn from(node: &impl WriterNode) -> Self {
    Self {
      tx: node.tx().clone(),
    }
  }
}

#[derive(ReaderNode, WriterNode)]
pub struct MockNode {
  btx: Btx,
  tx: Tx,
}

impl MockNode {
  pub fn new(btx: Btx, tx: Tx) -> Self {
    Self { btx, tx }
  }
}
