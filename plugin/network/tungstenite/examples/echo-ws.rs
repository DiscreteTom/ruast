use rua::model::Result;
use rua::node::{BcNode, Ctrlc, StdioNode};
use rua_tungstenite::listener::WsListener;

#[tokio::main]
pub async fn main() -> Result<()> {
  // broadcaster
  let bc = BcNode::default()
    .propagate_stop(true) // stop bc will stop all targets
    .spawn();

  // stdio
  StdioNode::default()
    .publish(&bc) // stdin => bc
    .subscribe(&bc) // bc => stdout
    .spawn();

  // websocket listener at 127.0.0.1:8080
  {
    let bc = bc.clone();
    WsListener::default()
      .peer_handler(move |ws_node| {
        ws_node
          .publish(&bc) // ws => bc
          .subscribe(&bc) // bc => ws
          .spawn();
      })
      .spawn()
      .await?;
    println!("WebSocket listener is running at ws://127.0.0.1:8080");
  }

  Ctrlc::new().publish(&bc).wait().await;

  Ok(())
}
