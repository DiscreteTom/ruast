use rua::{controller::EventHub, model::EventType, peer::StdioPeer};

fn main() {
  let s = EventHub::new();
  s.add_peer(StdioPeer::new(0, s.tx())).unwrap();
  loop {
    match s.recv() {
      EventType::PeerMsg(msg) => s.echo(msg).unwrap(),
      _ => break,
    }
  }
}
