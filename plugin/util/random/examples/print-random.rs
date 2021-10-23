use rua::{
  model::Result,
  node::{ctrlc::Ctrlc, Bc, StdioNode},
};
use rua_random::RandomNode;

#[tokio::main]
pub async fn main() -> Result<()> {
  // use stdout
  let stdio = StdioNode::new(16).spawn();

  // generate random alphanumeric bytes
  let random = RandomNode::new(16)
    .sink(stdio.clone()) // random => stdout
    .spawn()
    .unwrap();

  let mut bc = Bc::new(16);
  bc.add_target(stdio).await;
  bc.add_target(random).await;

  // broadcast NodeEvent::Stop
  Ctrlc::new().sink(bc.tx().clone()).wait().await;

  Ok(())
}
