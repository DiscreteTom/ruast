use bytes::BytesMut;
use rua::node::lockstep::Lockstep;
use rua::node::state::StateNode;
use rua::node::{ctrlc::Ctrlc, stdio::StdioNode};

#[tokio::main]
pub async fn main() {
  let state = StateNode::default(BytesMut::new()).spawn();

  let stdio = StdioNode::default()
    .on_msg({
      let state = state.clone();
      move |msg| state.apply(move |buffer| buffer.extend_from_slice(&msg))
    })
    .spawn();

  let ls = Lockstep::new()
    .step_length_ms(1000)
    .on_step({
      let state = state.clone();
      let stdio = stdio.clone();
      move |step| {
        state.apply({
          let stdio = stdio.clone();
          move |state| {
            let mut buffer = BytesMut::new();
            // append current step number
            buffer.extend_from_slice(&(step.to_string() + ":\n").into_bytes());
            buffer.extend_from_slice(state);
            state.clear();
            stdio.write(buffer.freeze())
          }
        })
      }
    })
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
