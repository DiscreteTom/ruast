use ctrlc;
use rua::{controller::EventHub, model::ServerEvent, peer::StdioPeer};

fn main() {
  let s = EventHub::new();
  s.add_peer(StdioPeer::new(0, s.tx())).unwrap();

  let tx = s.tx();
  ctrlc::set_handler(move || tx.send(ServerEvent::Stop).unwrap()).unwrap();

  loop {
    match s.recv() {
      ServerEvent::PeerMsg(msg) => s.echo(msg).unwrap(),
      _ => break,
    }
  }
}
