use clonesure::cc;
use rua::node::{ctrlc::Ctrlc, stdio::StdioNode};
use rua_random::RandomNode;

#[tokio::main]
pub async fn main() {
  // use stdout
  let stdio = StdioNode::default().spawn();

  // generate random alphanumeric bytes
  let rand = RandomNode::default()
    .on_msg(cc!(|@stdio, data| stdio.write(data)))
    .spawn()
    .unwrap();

  Ctrlc::default()
    .on_signal(move || {
      stdio.stop();
      rand.stop();
    })
    .wait()
    .await
    .expect("failed to listen for ctrlc");
}
