#[macro_export]
macro_rules! impl_node {
  (tx) => {
    fn tx(&self) -> &Tx {
      &self.tx
    }
  };

  (brx) => {
    fn brx(&self) -> Brx {
      self.btx.subscribe()
    }
  };

  (publish) => {
    /// self.btx => other.tx
    pub fn publish(self, other: &impl WriterNode) -> Self {
      let mut brx = self.brx();
      let tx = other.tx().clone();

      tokio::spawn(async move {
        loop {
          match brx.recv().await {
            Ok(e) => {
              if tx.send(e).await.is_err() {
                break;
              }
            }
            Err(_) => break,
          }
        }
      });
      self
    }
  };


  (subscribe) => {
    /// other.btx => self.tx
    pub fn subscribe(self, other: &impl ReaderNode) -> Self {
      let mut brx = other.brx();
      let tx = self.tx().clone();

      tokio::spawn(async move {
        loop {
          match brx.recv().await {
            Ok(e) => {
              if tx.send(e).await.is_err() {
                break;
              }
            }
            Err(_) => break,
          }
        }
      });
      self
    }
  };

  ($($i:ident),+)=>{
    $(impl_node!($i);)*
  }
}
