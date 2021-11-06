use bytes::Bytes;
use clonesure::cc;
use rua::node::{ctrlc::Ctrlc, file::FileNode, stdio::StdioNode};

#[tokio::main]
pub async fn main() {
  let file = FileNode::default()
    .filename("log.txt")
    .spawn()
    .await
    .expect("failed to create file peer");

  let stdio_node = StdioNode::default();
  let stdio = stdio_node.handle().clone();
  stdio_node
    .on_input(cc!(|@stdio, @file, msg| {
      file.write_then(
        msg,
        cc!(|@stdio, result| {
          match result {
            Ok(_) => stdio.write(Bytes::from_static(b">> ok")),
            Err(_) => stdio.write(Bytes::from_static(b">> err")),
          }
        }))
    }))
    .spawn();

  Ctrlc::default()
    .on_signal(move || {
      stdio.stop();
      file.stop();
    })
    .wait()
    .await;
}
