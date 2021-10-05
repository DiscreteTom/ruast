use ctrlc;
use rua::{model::ServerEvent, peer::StdioPeer, server::EventDrivenServer};

fn main() {
  let mut s = EventDrivenServer::new();
  s.on_peer_msg(&|msg, s| s.echo(msg).unwrap());
  s.add_peer(StdioPeer::new(0, s.tx())).unwrap();

  let tx = s.tx();
  ctrlc::set_handler(move || tx.send(ServerEvent::Stop).unwrap()).unwrap();
  s.start();
}
