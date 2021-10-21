use rua::model::{PeerEvent, Result};
use rua::peer::FilePeer;
use rua::{broadcaster::Broadcaster, peer::StdioPeer};

#[tokio::main]
pub async fn main() -> Result<()> {
  // Create a new broadcaster
  let mut bc = Broadcaster::new(16);

  // Add StdioPeer to Broadcaster
  bc.add_target({
    let mut stdio = StdioPeer::new(16);
    let tx = stdio.tx().clone();
    stdio.sink(bc.tx().clone()); // send to broadcaster
    stdio.spawn(); // start StdioPeer
    tx
  })
  .await;

  // Add FilePeer to Broadcaster
  bc.add_target({
    let mut file = FilePeer::new(16);
    let tx = file.tx().clone();
    file.filename("log.txt".to_string());
    file.spawn().await.expect("build FilePeer failed");
    tx
  })
  .await;

  // Wait for Ctrl-C
  tokio::signal::ctrl_c().await.unwrap();

  // Broadcast `PeerEvent::Stop`
  bc.tx().send(PeerEvent::Stop).await.ok();

  // Stop Broadcaster
  bc.stop().await;

  Ok(())
}
