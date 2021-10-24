use rua::node::{ctrlc::Ctrlc, FileNode, StdioNode};

#[tokio::main]
pub async fn main() {
  let stdio = StdioNode::default()
    .echo() // stdin => stdout
    .spawn();

  let file = FileNode::default()
    .filename("log.txt".to_string())
    .subscribe(&stdio) // => stdin => file
    .spawn()
    .await
    .expect("build FileNode failed");

  Ctrlc::new().publish(&stdio).publish(&file).wait().await;
}
