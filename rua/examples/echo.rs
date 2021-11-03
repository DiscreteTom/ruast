use clonesure::cc;
use rua::{
  model::{Stoppable, Writable},
  node::{ctrlc::Ctrlc, stdio::StdioNode},
};

#[tokio::main]
pub async fn main() {
  let stdio = StdioNode::new(16);
  let handle = stdio.handle();

  stdio
    .on_msg(cc!(|@handle, msg| handle.write(msg).unwrap()))
    .spawn();

  Ctrlc::new().on_signal(move || handle.stop()).wait().await;
}
