use rua::{
  model::Result,
  node::{ctrlc::Ctrlc, Broadcaster, FileNode, StdioNode},
};

#[tokio::main]
pub async fn main() -> Result<()> {
  // create a new broadcaster
  let mut bc = Broadcaster::new(16);

  // add StdioNode to broadcaster
  bc.add_target(
    StdioNode::new(16)
      .sink(bc.tx().clone()) // stdin => broadcaster
      .spawn(), // spawn reader & writer, return handle
  ) // broadcaster => stdout
  .await;

  // add FileNode to broadcaster
  bc.add_target(
    FileNode::new(16)
      .filename("log.txt".to_string())
      .spawn()
      .await
      .expect("build FileNode failed"),
  ) // broadcaster => file
  .await;

  // wait for ctrl-c, then broadcast NodeEvent::Stop
  Ctrlc::new().sink(bc.tx().clone()).wait().await;

  Ok(())

  // broadcaster will drop itself
}
