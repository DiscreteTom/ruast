#[derive(Default)]
pub struct Ctrlc {
  handler: Option<Box<dyn FnOnce() + Send>>,
}

impl Ctrlc {
  pub fn on_signal<F>(mut self, f: F) -> Self
  where
    F: FnOnce() + Send + 'static,
  {
    self.handler = Some(Box::new(f));
    self
  }

  pub fn spawn(self) {
    tokio::spawn(async move { self.wait().await });
  }

  pub async fn wait(self) {
    tokio::signal::ctrl_c()
      .await
      .expect("failed to listen for ctrlc");

    if let Some(handler) = self.handler {
      (handler)();
    }
  }
}
