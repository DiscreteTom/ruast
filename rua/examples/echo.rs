use rua::{
  model::Result,
  node::{ctrlc::Ctrlc, StdioNode},
};

#[tokio::main]
pub async fn main() -> Result<()> {
  // create & run StdioNode
  let stdio = StdioNode::new(16)
    .echo() // set sink to self
    .spawn(); // spawn reader & writer, return tx for StdioNode

  // wait for ctrl-c, then stop node
  Ctrlc::new().sink(stdio).wait().await;

  Ok(())
}
