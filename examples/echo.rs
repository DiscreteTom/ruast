use rua::{model::GameServer, peer, server::EventDrivenServer};

fn main() {
  let s = EventDrivenServer::new();
  s.new_peer(peer::StdioPeer::new).unwrap();
  s.start();
}
