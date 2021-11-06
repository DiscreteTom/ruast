use clonesure::cc;
use rua::node::{ctrlc::Ctrlc, stdio::StdioNode, tcp::TcpListener, Broadcaster};

/// Use `nc localhost 8080` to connect to the tcp server.
#[tokio::main]
pub async fn main() {
  let mut bc = Broadcaster::default();

  TcpListener::bind("127.0.0.1:8080")
    .on_new_peer(cc!(|@mut bc, node| {
      // new node will be added to the broadcaster
      bc.add_target(
        node
          .on_input(
            // new message will be sent to the broadcaster
            cc!(|@bc, data| bc.write(data))
          )
          .spawn())
    }))
    .spawn()
    .await
    .unwrap();

  // also print to stdout
  bc.add_target(StdioNode::default().spawn());

  Ctrlc::default()
    .on_signal(move || bc.stop_all())
    .wait()
    .await
    .expect("failed to listen for ctrlc");
}
