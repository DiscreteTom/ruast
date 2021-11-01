use rua::{
  model::{Stoppable, Writable},
  node::{ctrlc::Ctrlc, file::FileNode, stdio::StdioNode},
};

#[tokio::main]
pub async fn main() {
  let file = FileNode::default()
    .filename("log.txt".to_string())
    .spawn()
    .await
    .expect("failed to create file peer");

  let stdio = StdioNode::default()
    .on_msg({
      let file = file.clone();
      move |msg| file.write(msg).unwrap()
    })
    .spawn();

  Ctrlc::new()
    .on_signal(move || {
      stdio.stop();
      file.stop()
    })
    .wait().await;
}
