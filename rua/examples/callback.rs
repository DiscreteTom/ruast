use bytes::Bytes;
use clonesure::cc;
use rua::node::{ctrlc::Ctrlc, file::FileNode, stdio::StdioNode};
use rua_random::RandomNode;

#[tokio::main]
pub async fn main() {
  let file = FileNode::default()
    .filename("log.txt")
    .spawn()
    .await
    .expect("failed to create file peer");

  let stdio = StdioNode::default().spawn();

  let random = RandomNode::default()
    .on_msg(cc!(|@stdio, @file, msg| {
      file.write_then(
        msg,
        cc!(|@stdio, result| {
          match result {
            Ok(_) => stdio.write(Bytes::from_static(b"ok")),
            Err(_) => stdio.write(Bytes::from_static(b"err")),
          }
        }))
    }))
    .spawn()
    .unwrap();

  Ctrlc::default()
    .on_signal(move || {
      file.stop_then(cc!(|@random, @stdio, _| {
        random.clone().stop_then(cc!(|@stdio, _| {
          stdio.clone().stop();
        }))
      }));
    })
    .wait()
    .await
    .expect("failed to listen for ctrlc");
}
