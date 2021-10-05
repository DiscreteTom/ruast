use std::error::Error;

use rua::{
  peer::{FilePeer, StdioPeer},
  server::EventDrivenServer,
};

fn main() -> Result<(), Box<dyn Error>> {
  let mut s = EventDrivenServer::new();
  s.on_peer_msg(&|msg, s| {
    s.broadcast_all(msg.data);
  });

  s.add_peer(StdioPeer::new(0, s.tx()))?;
  s.add_peer(FilePeer::new(1, "log.txt")?)?;
  s.start();

  Ok(())
}
