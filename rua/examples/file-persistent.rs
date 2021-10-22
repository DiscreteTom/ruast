use rua::model::{NodeEvent, Result};
use rua::node::FileNode;
use rua::{broadcaster::Broadcaster, node::StdioNode};

#[tokio::main]
pub async fn main() -> Result<()> {
  // Create a new broadcaster
  let mut bc = Broadcaster::new(16);

  // Add StdioNode to Broadcaster
  bc.add_target(
    StdioNode::new(16).sink(bc.tx().clone()).spawn(), // start StdioNode
  )
  .await;

  // Add FileNode to Broadcaster
  bc.add_target(
    FileNode::new(16)
      .filename("log.txt".to_string())
      .spawn()
      .await
      .expect("build FileNode failed"),
  )
  .await;

  // Wait for Ctrl-C
  tokio::signal::ctrl_c().await.unwrap();

  // Broadcast `NodeEvent::Stop`
  bc.tx().send(NodeEvent::Stop).await.ok();

  Ok(())
}
