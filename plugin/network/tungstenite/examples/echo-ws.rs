use rua::model::Result;
use rua::node::{Bc, Ctrlc, StdioNode};
use rua_tungstenite::listener::WsListener;

const WS_LISTENER_ADDR: &str = "127.0.0.1:8080";

#[tokio::main]
pub async fn main() -> Result<()> {
  let mut bc = Bc::new(16);

  bc.add_target(
    StdioNode::new(16)
      .sink(bc.tx().clone()) // stdin => broadcaster
      .spawn(),
  ) // broadcaster => stdout
  .await;

  Ctrlc::new().sink(bc.tx().clone());

  let mut peer_rx = WsListener::new(WS_LISTENER_ADDR, 16, 16).spawn().await?;

  println!("WebSocket listener is running at ws://{}", WS_LISTENER_ADDR);

  loop {
    match peer_rx.recv().await {
      Some(p) => {
        bc.add_target(
          p.sink(bc.tx().clone()) // ws => broadcaster
            .spawn()
            .await?,
        ) // broadcaster => ws
        .await
      }
      None => todo!(),
    }
  }

  Ok(())
}
