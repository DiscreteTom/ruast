use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{go, take_mut};

pub struct StateNode<T: Send> {
  state: T,
  f_rx: Receiver<Box<dyn FnOnce(&mut T) + Send>>,
  handle: StateNodeHandle<T>,
}

impl<T: Send + 'static> StateNode<T> {
  pub fn new(state: T, buffer: usize) -> Self {
    let (f_tx, f_rx) = mpsc::channel(buffer);
    Self {
      state,
      f_rx,
      handle: StateNodeHandle { f_tx },
    }
  }

  pub fn with_state(state: T) -> Self {
    Self::new(state, 16)
  }

  pub fn handle(&self) -> &StateNodeHandle<T> {
    &self.handle
  }

  pub fn spawn(self) -> StateNodeHandle<T> {
    take_mut!(self, state, f_rx);
    go! {
      while let Some(f) = f_rx.recv().await {
        f(&mut state);
      }
    };
    self.handle
  }
}

#[derive(Clone)]
pub struct StateNodeHandle<T> {
  f_tx: Sender<Box<dyn FnOnce(&mut T) + Send>>,
}

impl<T: 'static> StateNodeHandle<T> {
  pub fn apply<F>(&self, f: F)
  where
    F: FnOnce(&mut T) + 'static + Send,
  {
    let f_tx = self.f_tx.clone();
    tokio::spawn(async move { f_tx.send(Box::new(f)).await });
  }
}
