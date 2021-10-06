use rua::{
  controller::{lockstep::LockstepController, EventHub},
  model::Event,
  peer::StdioPeer,
};

fn main() {
  let mut peer_msgs = Vec::new();
  let s = EventHub::new();

  let lockstep_op_code = 0;
  let mut lockstepper = LockstepController::new(1000, s.tx(), lockstep_op_code);
  lockstepper.next_step();

  s.add_peer(StdioPeer::new(0, s.tx())).unwrap();

  loop {
    match s.recv() {
      Event::PeerMsg(msg) => peer_msgs.push(msg.data),
      Event::Custom(code) => {
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
