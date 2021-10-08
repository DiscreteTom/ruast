use rua::{controller::EventHub, model::Event, peer::StdioPeer};

#[tokio::main]
pub async fn main() {
  let mut h = EventHub::new(256);
  h.add_peer(StdioPeer::new(0, h.tx_clone(), 256)).unwrap();
  loop {
    match h.recv().await {
      Event::PeerMsg(msg) => h.echo(msg).await.unwrap(),
      _ => break,
    }
  }
}
