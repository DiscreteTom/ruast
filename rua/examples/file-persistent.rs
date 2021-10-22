use rua::model::{PeerEvent, Result};
use rua::node::FilePeer;
use rua::{broadcaster::Broadcaster, node::StdioPeer};

#[tokio::main]
pub async fn main() -> Result<()> {
  // Create a new broadcaster
  let mut bc = Broadcaster::new(16);

  // Add StdioPeer to Broadcaster
  bc.add_target(
    StdioPeer::new(16).sink(bc.tx().clone()).spawn(), // start StdioPeer
  )
  .await;

  // Add FilePeer to Broadcaster
  bc.add_target(
    FilePeer::new(16)
      .filename("log.txt".to_string())
      .spawn()
      .await
      .expect("build FilePeer failed"),
  )
  .await;

  // Wait for Ctrl-C
  tokio::signal::ctrl_c().await.unwrap();

  // Broadcast `PeerEvent::Stop`
  bc.tx().send(PeerEvent::Stop).await.ok();

  Ok(())
}
