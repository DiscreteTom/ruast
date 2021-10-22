use crate::{
  impl_node,
  model::{NodeEvent, Result, Tx},
};

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
/// let (tx, rx) = tokio::mpsc::channel(1);
/// Ctrlc::new().sink(tx).spawn().unwrap();
/// rx.recv().await;
/// ```
///
/// ## Wait for Ctrl-C then send NodeEvent
///
/// ```
/// let (tx, rx) = tokio::mpsc::channel(1);
/// Ctrlc::new().sink(tx).wait().await;
/// rx.recv().await;
/// ```
pub struct Ctrlc {
  sink: Option<Tx>,
}

impl Ctrlc {
  impl_node!(sink);

  pub fn new() -> Self {
    Self { sink: None }
  }

  /// You have to set `sink` before `spawn`.
  pub fn spawn(self) -> Result<()> {
    let sink = self.sink.ok_or("missing sink when spawn ctrlc")?;
    tokio::spawn(async move {
      tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrlc");
      sink
        .send(NodeEvent::Stop)
        .await
        .expect("ctrlc failed to send NodeEvent::Stop");
    });
    Ok(())
  }

  /// Wait for Ctrl-C. If `sink` exists, send `NodeEvent::Stop` to the sink.
  pub async fn wait(self) {
    tokio::signal::ctrl_c()
      .await
      .expect("failed to listen for ctrlc");

    if let Some(sink) = self.sink {
      sink
        .send(NodeEvent::Stop)
        .await
        .expect("ctrlc failed to send NodeEvent::Stop");
    }
  }
}
