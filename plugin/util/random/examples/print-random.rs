use rua::{
  model::Result,
  node::{ctrlc::Ctrlc, StdioNode},
};
use rua_random::RandomNode;

#[tokio::main]
pub async fn main() -> Result<()> {
  // use stdout
  let stdio = StdioNode::default().spawn();

  // generate random alphanumeric bytes
  RandomNode::default()
    .publish(&stdio) // random => stdout
    .spawn();

  // broadcast NodeEvent::Stop
  Ctrlc::new().publish(&stdio).wait().await;

  Ok(())
}
