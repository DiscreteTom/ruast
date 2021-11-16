use bytes::BytesMut;
use clonesure::cc;
use rua::node::{ctrlc::Ctrlc, state::StateNode, stdio::StdioNode, time::Ticker};

#[tokio::main]
pub async fn main() {
  let state = StateNode::with_state(BytesMut::new()).spawn();

  let stdio = StdioNode::default()
    .on_input(cc!(|@state, msg| state.apply(move |buffer| buffer.extend_from_slice(&msg))))
    .spawn();

  let ticker = Ticker::default()
    .on_tick(cc!(|@state, @stdio, tick| {
      state.apply(cc!(|@stdio, state| {
        let mut result = BytesMut::new();
        // append current tick number
        result.extend_from_slice(&(tick.to_string() + ":\n").into_bytes());
        // append state
        result.extend_from_slice(state);

        state.clear();
        stdio.write(result.freeze())
      }))
    }))
    .spawn()
    .expect("failed to start ticker");

  Ctrlc::default()
    .on_signal(move || {
      ticker.stop();
      stdio.stop();
    })
    .wait()
    .await
    .expect("failed to listen for ctrlc");
}
