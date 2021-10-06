use rua::{
  model::ServerEvent,
  peer::StdioPeer,
  server::{lockstep::LockstepController, EventDrivenServer},
};

fn main() {
  let mut peer_msgs = Vec::new();
  let s = EventDrivenServer::new();

  let lockstep_op_code = 0;
  let mut lockstepper = LockstepController::new(1000, s.tx(), lockstep_op_code);
  lockstepper.next_step();

  s.add_peer(StdioPeer::new(0, s.tx())).unwrap();

  loop {
    match s.recv() {
      ServerEvent::PeerMsg(msg) => peer_msgs.push(msg.data),
      ServerEvent::Custom(code) => {
        if code == lockstep_op_code {
          for data in &peer_msgs {
            s.broadcast_all(data.clone());
          }
          peer_msgs.clear();
          lockstepper.next_step();
        }
      }
      _ => break,
    }
  }
}
