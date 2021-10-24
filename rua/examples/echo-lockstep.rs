use bytes::BytesMut;
use rua::node::{Ctrlc, LsNode, StdioNode};

#[tokio::main]
pub async fn main() {
  let stdio = StdioNode::default().spawn();

  let ls = LsNode::default_with_step_length(1000)
    .reducer(|current_step, data| {
      let mut buffer = BytesMut::new();
      // append current step number
      buffer.extend_from_slice(&(current_step.to_string() + ":\n").into_bytes());
      // append data during this step
      for d in data {
        buffer.extend_from_slice(&d);
      }
      buffer.freeze()
    })
    .subscribe(&stdio) // stdin => lockstep
    .publish(&stdio) // lockstep => stdout
    .spawn();

  Ctrlc::new().publish(&stdio).publish(&ls).wait().await;
}
