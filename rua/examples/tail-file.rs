use clonesure::cc;
use rua::node::{ctrlc::Ctrlc, stdio::StdioNode, tail::TailNode};

#[tokio::main]
pub async fn main() {
  let stdio = StdioNode::default().spawn();
  let tail = TailNode::with_file_name("log.txt")
    .on_new_line(cc!(|@stdio, data| stdio.write(data)))
    .spawn()
    .await
    .expect("failed to tail file");

  Ctrlc::default()
    .on_signal(move || {
      stdio.stop();
      tail.stop()
    })
    .wait()
    .await
    .expect("failed to listen for ctrlc");
}
