use rua::node::{ctrlc::Ctrlc, StdioNode};

#[tokio::main]
pub async fn main() {
  let stdio = StdioNode::default()
    .echo() // stdin => stdout
    .spawn();

  Ctrlc::new().publish(&stdio).wait().await;
}
