use rua::{model::GameServer, peer, server::EventDrivenServer};

fn main() {
  let mut s = EventDrivenServer::new();
  s.on_peer_msg(|msg| {
    (msg.peer.upgrade().unwrap())
      .lock()
      .unwrap()
      .write(msg.data.clone())
      .unwrap();
  });
  s.new_peer(peer::StdioPeer::new).unwrap();
  s.start();
}
