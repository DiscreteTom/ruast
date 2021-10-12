use rua::{
  controller::EventHub,
  model::{HubEvent, PeerBuilder, Result},
  peer::StdioPeerBuilder,
};
use rua_tungstenite::listener::WebsocketListener;

const WS_LISTENER_ADDR: &str = "127.0.0.1:8080";

#[tokio::main]
pub async fn main() -> Result<()> {
  let mut current_peer_id = 0;

  let mut h = EventHub::new(256);
  h.add_peer(
    StdioPeerBuilder::new()
      .id(current_peer_id)
      .buffer(32)
      .hub_tx(h.tx_clone())
      .build()?,
  )?;
  current_peer_id += 1;

  let ws_listener_code = 0;
  let (ws_listener, mut ws_peer_rx) =
    WebsocketListener::new(WS_LISTENER_ADDR, ws_listener_code, h.tx_clone(), 256);
  tokio::spawn(async move { ws_listener.start().await });

  println!("WebSocket listener is running at ws://{}", WS_LISTENER_ADDR);

  loop {
    match h.recv().await {
      HubEvent::PeerMsg(msg) => {
        h.broadcast_all(msg.data).await;
      }
      HubEvent::Custom(code) => {
        if code == ws_listener_code {
          h.add_peer(
            ws_peer_rx
              .recv()
              .await
              .unwrap()
              .id(current_peer_id)
              .hub_tx(h.tx_clone())
              .buffer(32)
              .build()
              .await,
          )?;
          current_peer_id += 1;
        }
      }
      HubEvent::RemovePeer(id) => h.remove_peer(id).await?,
      _ => break,
    }
  }

  Ok(())
}
