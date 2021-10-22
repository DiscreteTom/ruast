use rua::{
  model::{NodeEvent, Result},
  node::StdioNode,
};

#[tokio::main]
pub async fn main() -> Result<()> {
  // create & run StdioNode
  let handle = StdioNode::new(16)
    .echo() // set sink to self
    .spawn(); // spawn reader & writer, return handle

  // wait for ctrl-c
  tokio::signal::ctrl_c().await.unwrap();

  // stop node
  handle.send(NodeEvent::Stop).await.ok();

  Ok(())
}
