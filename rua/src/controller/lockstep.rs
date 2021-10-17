use std::time::Duration;
use tokio::{sync::mpsc::Sender, time};

use crate::model::ServerEvent;

pub struct LockstepController {
  step_length: u64, // in ms
  current_step: u64,
  server_tx: Sender<ServerEvent>,
  op_code: u32,
}

impl LockstepController {
  pub fn new(step_length: u64, server_tx: Sender<ServerEvent>, op_code: u32) -> Self {
    LockstepController {
      step_length,
      server_tx,
      op_code,
      current_step: 0,
    }
  }

  pub fn set_step_length(&mut self, step_length: u64) {
    self.step_length = step_length
  }

  pub fn current_step(&self) -> u64 {
    self.current_step
  }

  pub fn next_step(&mut self) {
    self.current_step += 1;
    let step_length = self.step_length;
    let server_tx = self.server_tx.clone();
    let op_code = self.op_code;
    tokio::spawn(async move {
      time::sleep(Duration::from_millis(step_length)).await;
      server_tx.send(ServerEvent::Custom(op_code)).await.unwrap();
    });
  }
}
