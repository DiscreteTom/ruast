use std::error::Error;

use rua::{model::GameServer, peer::StdioPeer, server::EventDrivenServer};

fn main() -> Result<(), Box<dyn Error>> {
  let mut s = EventDrivenServer::new();
  s.on_peer_msg(&|msg, s| s.write_to(msg.peer_id, msg.data).unwrap());
  s.add_peer(StdioPeer::new(0, s.tx())).unwrap();
  s.start();
  Ok(())
}
