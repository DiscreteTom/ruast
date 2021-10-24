use super::mock::MockReaderNode;
use crate::impl_node;
use crate::model::{Brx, Btx, NodeEvent, ReaderNode, WriterNode};
use rua_macro::ReaderNode;
use tokio::sync::broadcast::channel;

/// Ctrl-C handler.
///
/// # Examples
///
/// ## Wait for Ctrl-C
///
/// ```
/// Ctrlc::new().wait().await;
/// ```
///
/// ## Async wait and broadcast NodeEvent
///
/// ```
/// // register targets when constructing ctrlc
/// let ctrlc = Ctrlc::new().publish(&node1).publish(&node2).spawn();
/// // register targets after constructing ctrlc
/// node3.subscribe(&ctrlc);
/// node4.subscribe(&ctrlc);
/// ```
///
/// ## Wait for Ctrl-C then broadcast NodeEvent
///
/// ```
/// Ctrlc::new().publish(&node1).publish(&node2).wait().await;
/// ```
#[derive(ReaderNode)]
pub struct Ctrlc {
  btx: Btx,
}

impl Ctrlc {
  pub fn new() -> Self {
    let (btx, _) = channel(1);
    Self { btx }
  }

  pub fn spawn(self) -> MockReaderNode {
    let btx = self.btx.clone();
    tokio::spawn(async move {
      tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrlc");
      btx.send(NodeEvent::Stop).ok();
    });
    MockReaderNode::new(self.btx)
  }

  pub async fn wait(self) {
    tokio::signal::ctrl_c()
      .await
      .expect("failed to listen for ctrlc");

    self.btx.send(NodeEvent::Stop).ok();
  }
}
