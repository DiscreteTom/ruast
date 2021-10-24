use bytes::BytesMut;
use rua::model::Result;
use rua::node::{Ctrlc, LsNode, StdioNode};

#[tokio::main]
pub async fn main() -> Result<()> {
  let stdio = StdioNode::default().spawn();

  let ls = LsNode::default(1000, BytesMut::new())
    .on_msg(|_, data, state| {
      state.extend_from_slice(&data);
    })
    .on_step(|current_step, state| {
      let mut buffer = BytesMut::new();
      // append current step number
      buffer.extend_from_slice(&(current_step.to_string() + ":\n").into_bytes());
      buffer.extend_from_slice(state);
      state.clear();
      buffer.freeze()
    })
    .subscribe(&stdio) // stdin => lockstep
    .publish(&stdio) // lockstep => stdout
    .spawn()?;

  Ctrlc::new().publish(&stdio).publish(&ls).wait().await;

  Ok(())
}
