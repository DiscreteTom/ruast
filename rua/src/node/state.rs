use tokio::sync::mpsc::{self, Receiver, Sender};

pub struct StateNode<T: Send> {
  state: T,
  f_rx: Receiver<Box<dyn FnOnce(&mut T) + Send>>,
  handle: StateNodeHandle<T>,
}

impl<T: 'static + Send> StateNode<T> {
  pub fn new(state: T, buffer: usize) -> Self {
    let (f_tx, f_rx) = mpsc::channel(buffer);
    Self {
      state,
      f_rx,
      handle: StateNodeHandle::new(f_tx),
    }
  }

  pub fn default(state: T) -> Self {
    Self::new(state, 16)
  }

  pub fn spawn(self) -> StateNodeHandle<T> {
    let mut state = self.state;
    let mut f_rx = self.f_rx;
    tokio::spawn(async move {
      while let Some(f) = f_rx.recv().await {
        f(&mut state);
      }
    });
    self.handle
  }
}

#[derive(Clone)]
pub struct StateNodeHandle<T> {
  f_tx: Sender<Box<dyn FnOnce(&mut T) + Send>>,
}

impl<T: 'static> StateNodeHandle<T> {
  fn new(f_tx: Sender<Box<dyn FnOnce(&mut T) + Send>>) -> Self {
    Self { f_tx }
  }

  pub fn apply(&self, f: impl FnOnce(&mut T) + 'static + Send) {
    let f_tx = self.f_tx.clone();
    tokio::spawn(async move { f_tx.send(Box::new(f)).await });
  }
}
