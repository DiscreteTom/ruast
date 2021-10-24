use crate::impl_node;
use crate::model::{Brx, Btx, ReaderNode, Tx, WriterNode};
use rua_macro::{ReaderNode, WriterNode};

#[derive(ReaderNode, Debug, Clone)]
pub struct MockReaderNode {
  btx: Btx,
}

impl MockReaderNode {
  pub fn new(btx: Btx) -> Self {
    Self { btx }
  }
}

#[derive(WriterNode, Debug, Clone)]
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

#[derive(ReaderNode, WriterNode, Debug, Clone)]
pub struct MockNode {
  btx: Btx,
  tx: Tx,
}

impl MockNode {
  pub fn new(btx: Btx, tx: Tx) -> Self {
    Self { btx, tx }
  }
}
