use clonesure::cc;
use rua::node::{ctrlc::Ctrlc, stdio::StdioNode};

#[tokio::main]
pub async fn main() {
  let stdio = StdioNode::default();
  let handle = stdio.handle().clone();

  stdio
    .on_input(cc!(|@handle, msg| handle.write(msg)))
    .spawn();

  Ctrlc::new().on_signal(move || handle.stop()).wait().await;
}
