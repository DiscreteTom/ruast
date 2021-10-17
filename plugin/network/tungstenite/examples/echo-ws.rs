use rua::{
  controller::PeerManager,
  model::{PeerBuilder, Result, ServerEvent},
  peer::StdioPeerBuilder,
};
use rua_tungstenite::listener::WebsocketListener;
use tokio::sync::mpsc;

const WS_LISTENER_ADDR: &str = "127.0.0.1:8080";

#[tokio::main]
pub async fn main() -> Result<()> {
  let mut current_peer_id = 0;

  let mut h = PeerManager::new();
  let (tx, mut rx) = mpsc::channel(256);

  h.add_peer(
    StdioPeerBuilder::new()
      .id(current_peer_id)
      .server_tx(tx.clone())
      .build()
      .await?,
  )?;
  current_peer_id += 1;

  let ws_listener_code = 0;
  let (ws_listener, mut ws_peer_rx) =
    WebsocketListener::new(WS_LISTENER_ADDR, ws_listener_code, tx.clone(), 256);
  tokio::spawn(async move { ws_listener.start().await });

  println!("WebSocket listener is running at ws://{}", WS_LISTENER_ADDR);

  loop {
    match rx.recv().await.unwrap() {
      ServerEvent::PeerMsg(msg) => {
        h.broadcast_all(msg.data).await;
      }
      ServerEvent::Custom(code) => {
        if code == ws_listener_code {
          h.add_peer(
            ws_peer_rx
              .recv()
              .await
              .unwrap()
              .id(current_peer_id)
              .server_tx(tx.clone())
              .build()
              .await?,
          )?;
          current_peer_id += 1;
        }
      }
      ServerEvent::RemovePeer(id) => h.remove_peer(id)?,
      _ => break,
    }
  }

  Ok(())
}
