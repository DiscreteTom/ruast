use bytes::Bytes;
use clonesure::cc;
use rua::node::{ctrlc::Ctrlc, stdio::StdioNode, tail::TailNode, FileNode, Ticker};

#[tokio::main]
pub async fn main() {
  const FILENAME: &str = "log.txt";

  // write file
  let file = FileNode::default()
    .filename(FILENAME)
    .spawn()
    .await
    .expect("failed to write file");
  let ticker = Ticker::default()
    .on_tick(cc!(|@file, tick| file.write(Bytes::from(tick.to_string()))))
    .spawn()
    .unwrap();

  // tail file to stdout
  let stdio = StdioNode::default().spawn();
  let tail = TailNode::with_file_name(FILENAME)
    .on_new_line(cc!(|@stdio, data| stdio.write(data)))
    .spawn()
    .await
    .expect("failed to tail file");

  Ctrlc::default()
    .on_signal(move || {
      stdio.stop();
      tail.stop();
      ticker.stop();
      file.stop();
    })
    .wait()
    .await
    .expect("failed to listen for ctrlc");
}
