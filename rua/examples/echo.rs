use rua::{
  model::{Stoppable, Writable},
  node::{ctrlc::Ctrlc, stdio::StdioNode},
};

#[tokio::main]
pub async fn main() {
  let stdio = StdioNode::new(16);
  let stdio_handle = stdio.handle();
  let stdio = stdio.on_msg(move |msg| stdio_handle.write(msg));
  let stdio_handle = stdio.spawn();

  Ctrlc::new()
    .on_signal(move || stdio_handle.stop())
    .wait()
    .await;
}
