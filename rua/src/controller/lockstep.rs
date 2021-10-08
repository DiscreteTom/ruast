use std::time::Duration;
use tokio::{sync::mpsc::Sender, time};

use crate::model::HubEvent;

pub struct LockstepController {
  step_length: u64, // in ms
  current_step: u64,
  hub_tx: Sender<HubEvent>,
  op_code: u32,
}

impl LockstepController {
  pub fn new(step_length: u64, hub_tx: Sender<HubEvent>, op_code: u32) -> Self {
    LockstepController {
      step_length,
      hub_tx,
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
    let hub_tx = self.hub_tx.clone();
    let op_code = self.op_code;
    tokio::spawn(async move {
      time::sleep(Duration::from_millis(step_length)).await;
      hub_tx.send(HubEvent::Custom(op_code)).await.unwrap();
    });
  }
}
