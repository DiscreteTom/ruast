use std::error::Error;

use rua::{model::GameServer, peer::StdioPeer, server::EventDrivenServer};

fn main() -> Result<(), Box<dyn Error>> {
  let (mut s, event_sender) = EventDrivenServer::new();
  s.on_peer_msg(|msg| {
    (msg.peer.upgrade().unwrap())
      .lock()
      .unwrap()
      .write(msg.data.clone())
      .unwrap();
  });
  s.add_peer(StdioPeer::new(String::from("stdio"), event_sender.clone())?)
    .unwrap();
  s.start();
  Ok(())
}
