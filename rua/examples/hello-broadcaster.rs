use clonesure::cc;
use rua::node::{ctrlc::Ctrlc, file::FileNode, stdio::StdioNode, Broadcaster};

#[tokio::main]
pub async fn main() {
  let mut bc = Broadcaster::default();

  bc.add_target(
    FileNode::default()
      .filename("log.txt")
      .spawn()
      .await
      .expect("failed to create file peer"),
  );

  bc.add_target(
    StdioNode::default()
      .on_input(cc!(|@bc, msg| bc.write(msg)))
      .spawn(),
  );

  Ctrlc::default()
    .on_signal(move || bc.stop_all())
    .wait()
    .await
    .expect("failed to listen for ctrlc");
}
