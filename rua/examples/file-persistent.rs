use clonesure::cc;
use rua::node::{ctrlc::Ctrlc, file::FileNode, stdio::StdioNode};

#[tokio::main]
pub async fn main() {
  let file = FileNode::default()
    .filename("log.txt")
    .spawn()
    .await
    .expect("failed to create file peer");

  let stdio = StdioNode::default()
    .on_input(cc!(|@file, msg| file.write(msg)))
    .spawn();

  Ctrlc::default()
    .on_signal(move || {
      stdio.stop();
      file.stop()
    })
    .wait()
    .await
    .expect("failed to listen for ctrlc");
}
