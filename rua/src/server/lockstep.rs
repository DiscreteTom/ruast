use std::{thread, time::Duration};

pub struct LockstepController<'a> {
  step_length: u64, // in ms
  step_handler: &'a dyn Fn(&Self),
  current_step: u64,
}

impl<'a> LockstepController<'a> {
  pub fn new(step_length: u64) -> Self {
    LockstepController {
      step_length,
      current_step: 0,
      step_handler: &|_| {},
    }
  }

  pub fn set_step_length(&mut self, step_length: u64) {
    self.step_length = step_length
  }

  pub fn current_step(&self) -> u64 {
    self.current_step
  }

  pub fn on_step(&mut self, f: &'a dyn Fn(&Self)) -> &Self {
    self.step_handler = f;
    self
  }

  pub fn start(&mut self) {
    loop {
      thread::sleep(Duration::from_micros(self.step_length));
      self.current_step += 1;
      (self.step_handler)(self);
    }
  }
}
