use rua::model::{NodeEvent, Result};
use rua::node::StdioNode;

#[tokio::main]
pub async fn main() -> Result<()> {
  // Create & run StdioNode
  let stdio = StdioNode::new(16);
  let stdio_handle = stdio.echo().spawn(); // set sink to self, then spawn

  // Wait for Ctrl-C
  tokio::signal::ctrl_c().await.unwrap();

  // Stop StdioNode
  stdio_handle.send(NodeEvent::Stop).await.unwrap();

  Ok(())
}
