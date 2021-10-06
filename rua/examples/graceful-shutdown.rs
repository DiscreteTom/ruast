use ctrlc;
use rua::{model::ServerEvent, peer::StdioPeer, server::MsgHub};

fn main() {
  let s = MsgHub::new();
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
