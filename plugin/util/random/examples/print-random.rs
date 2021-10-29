use rua::{
  model::{Result, Stoppable, Writable},
  node::{ctrlc::Ctrlc, stdio::StdioNode},
};
use rua_random::RandomNode;

#[tokio::main]
pub async fn main() -> Result<()> {
  // use stdout
  let stdio = StdioNode::default().spawn();

  // generate random alphanumeric bytes
  let rand = RandomNode::new()
    .on_msg({
      let stdio = stdio.clone();
      move |data| stdio.write(data)
    })
    .spawn()
    .expect("failed to create RandomNode");

  Ctrlc::new()
    .on_signal(move || {
      stdio.stop();
      rand.stop();
    })
    .wait()
    .await;

  Ok(())
}
