use rua::model::Writable;
use rua::node::{broadcast::BcNode, Ctrlc, StdioNode};
use rua_tungstenite::listener::WsListener;

#[tokio::main]
pub async fn main() {
  // broadcaster
  let mut bc = BcNode::default();

  // stdio
  let stdio = StdioNode::default()
    .on_msg({
      let bc = bc.clone();
      move |data| bc.write(data)
    })
    .spawn();
  bc.add_target(stdio);

  // websocket listener at 127.0.0.1:8080
  WsListener::default()
    .on_new_peer({
      move |ws_node| {
        bc.add_target(ws_node.handle());
        ws_node
          .on_msg({
            let bc = bc.clone();
            move |data| bc.write(data)
          })
          .spawn();
      }
    })
    .spawn()
    .await
    .expect("WebSocket listener failed to bind address");

  println!("WebSocket listener is running at ws://127.0.0.1:8080");

  Ctrlc::new().wait().await;
}
