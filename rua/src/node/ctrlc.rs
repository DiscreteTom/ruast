use tokio::sync::broadcast::channel;

use crate::model::{Btx, NodeEvent};

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
/// ## Async wait and send NodeEvent
///
/// ```
/// let (tx, rx) = tokio::broadcast::channel(1);
/// Ctrlc::new().publish(tx).spawn().unwrap();
/// rx.recv().await;
/// ```
///
/// ## Wait for Ctrl-C then send NodeEvent
///
/// ```
/// let (tx, rx) = tokio::broadcast::channel(1);
/// Ctrlc::new().publish(tx).wait().await;
/// rx.recv().await;
/// ```
pub struct Ctrlc {
  btx: Btx,
}

impl Ctrlc {
  pub fn new() -> Self {
    let (btx, _) = channel(1);
    Self { btx }
  }

  pub fn publish(self, target: Btx) -> Self {
    let mut brx = self.btx.subscribe();
    tokio::spawn(async move {
      let e = brx.recv().await.unwrap(); // e is NodeEvent::Stop
      target.send(e).ok();
    });
    self
  }

  pub fn spawn(self) {
    let btx = self.btx;
    tokio::spawn(async move {
      tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrlc");
      btx.send(NodeEvent::Stop).ok();
    });
  }

  /// Wait for Ctrl-C. If `sink` exists, send `NodeEvent::Stop` to the sink.
  pub async fn wait(self) {
    tokio::signal::ctrl_c()
      .await
      .expect("failed to listen for ctrlc");

    self.btx.send(NodeEvent::Stop).ok();
  }
}
