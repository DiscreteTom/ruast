use rua::model::{PeerEvent, Result};
use rua::peer::StdioPeer;

#[tokio::main]
pub async fn main() -> Result<()> {
  // Create & run StdioPeer
  let mut stdio = StdioPeer::new(16);
  stdio.sink(stdio.tx().clone()); // echo to self
  let tx = stdio.spawn(); // run StdioPeer

  // Wait for Ctrl-C
  tokio::signal::ctrl_c().await.unwrap();

  // Stop StdioPeer
  tx.send(PeerEvent::Stop).await.unwrap();

  Ok(())
}
