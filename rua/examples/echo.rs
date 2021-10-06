use rua::{model::ServerEvent, peer::StdioPeer, server::MsgHub};

fn main() {
  let s = MsgHub::new();
  s.add_peer(StdioPeer::new(0, s.tx())).unwrap();
  loop {
    match s.recv() {
      ServerEvent::PeerMsg(msg) => s.echo(msg).unwrap(),
      _ => break,
    }
  }
}
