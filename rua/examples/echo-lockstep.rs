use bytes::BytesMut;
use clonesure::cc;
use rua::model::{Stoppable, Writable};
use rua::node::lockstep::Lockstep;
use rua::node::state::StateNode;
use rua::node::{ctrlc::Ctrlc, stdio::StdioNode};

#[tokio::main]
pub async fn main() {
  let state = StateNode::default(BytesMut::new()).spawn();

  let stdio = StdioNode::default()
    .on_msg(cc!(|@state, msg| state.apply(move |buffer| buffer.extend_from_slice(&msg))))
    .spawn();

  let ls = Lockstep::new()
    .step_length_ms(1000)
    .on_step(cc!(|@state, @stdio, step| {
      state.apply(cc!(|@stdio, state| {
        let mut buffer = BytesMut::new();
        // append current step number
        buffer.extend_from_slice(&(step.to_string() + ":\n").into_bytes());
        buffer.extend_from_slice(state);
        state.clear();
        stdio.write(buffer.freeze()).unwrap()
      }))
    }))
    .spawn()
    .expect("failed to start lockstep controller");

  Ctrlc::new()
    .on_signal(move || {
      ls.stop();
      stdio.stop()
    })
    .wait()
    .await;
}
