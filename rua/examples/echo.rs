use rua::{controller::EventHub, model::Event, peer::StdioPeer};

fn main() {
  let h = EventHub::new();
  h.add_peer(StdioPeer::new(0, h.tx())).unwrap();
  loop {
    match h.recv() {
      Event::PeerMsg(msg) => h.echo(msg).unwrap(),
      _ => break,
    }
  }
}
