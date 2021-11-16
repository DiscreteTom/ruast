use clonesure::cc;
use rua::node::{broadcast::Broadcaster, Ctrlc, StdioNode};
use rua_tungstenite::listener::WsListener;

/// Use `wscat -c ws://127.0.0.1:8080` to connect to the websocket server.
#[tokio::main]
pub async fn main() {
  // stoppable broadcaster
  let mut bc = Broadcaster::default();

  // websocket listener at 127.0.0.1:8080
  let ws = WsListener::bind("127.0.0.1:8080")
    .on_new_peer(cc!(|@mut bc, ws_node| {
      // new node will be added to the broadcaster
      bc.add_target(
        ws_node
          .on_msg(cc!(|@bc, data| bc.write(data)))
          .spawn()
      );
    }))
    .spawn()
    .await
    .expect("WebSocket listener failed to bind address");

  // also print to stdout
  bc.add_target(StdioNode::default().spawn());

  println!("WebSocket listener is running at ws://127.0.0.1:8080");

  // wait for ctrlc, stop all targets
  Ctrlc::default()
    .on_signal(move || {
      bc.stop_all();
      ws.stop()
    })
    .wait()
    .await
    .expect("failed to listen for ctrlc");
}
