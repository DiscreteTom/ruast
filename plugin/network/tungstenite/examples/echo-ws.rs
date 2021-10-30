use rua::model::{Stoppable, Writable};
use rua::node::broadcast::StoppableBcNode;
use rua::node::{Ctrlc, StdioNode};
use rua_tungstenite::listener::WsListener;

#[tokio::main]
pub async fn main() {
  // stoppable broadcaster
  // stop bc will stop all targets
  let mut bc = StoppableBcNode::default();

  // stdio
  let stdio = StdioNode::default()
    .on_msg({
      let bc = bc.clone();
      move |data| bc.write(data).unwrap()
    })
    .spawn();
  bc.add_target(stdio);

  // websocket listener at 127.0.0.1:8080
  WsListener::default()
    .on_new_peer({
      let mut bc = bc.clone();
      move |ws_node| {
        bc.add_target(ws_node.handle());
        ws_node
          .on_msg({
            let bc = bc.clone();
            move |data| bc.write(data).unwrap()
          })
          .spawn();
      }
    })
    .spawn()
    .await
    .expect("WebSocket listener failed to bind address");

  println!("WebSocket listener is running at ws://127.0.0.1:8080");

  // wait for ctrlc, stop all targets
  Ctrlc::new().on_signal(move || bc.stop()).wait().await;
}
