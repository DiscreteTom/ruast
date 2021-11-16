use bytes::BytesMut;
use clonesure::cc;
use rua::node::{ctrlc::Ctrlc, state::StateNode, stdio::StdioNode, time::Lockstep};

#[tokio::main]
pub async fn main() {
  let state = StateNode::with_state(BytesMut::new()).spawn();

  let stdio = StdioNode::default()
    .on_input(cc!(|@state, msg| state.apply(move |buffer| buffer.extend_from_slice(&msg))))
    .spawn();

  let ls = Lockstep::default()
    .on_step(cc!(|@state, @stdio, step| {
      state.apply(cc!(|@stdio, state| {
        let mut result = BytesMut::new();
        // append current step number
        result.extend_from_slice(&(step.to_string() + ":\n").into_bytes());
        // append state
        result.extend_from_slice(state);

        state.clear();
        stdio.write(result.freeze())
      }))
    }))
    .spawn()
    .expect("failed to start lockstep controller");

  Ctrlc::default()
    .on_signal(move || {
      ls.stop();
      stdio.stop();
    })
    .wait()
    .await
    .expect("failed to listen for ctrlc");
}
