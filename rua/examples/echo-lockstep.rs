use std::sync::Arc;

use rua::{
  controller::{EventHub, LockstepController},
  model::Event,
  peer::StdioPeer,
};

fn main() {
  let mut peer_msgs = Vec::new();
  let h = EventHub::new();

  let lockstep_op_code = 0;
  let mut lockstepper = LockstepController::new(1000, h.tx(), lockstep_op_code);
  lockstepper.next_step();

  h.add_peer(StdioPeer::new(0, h.tx())).unwrap();

  loop {
    match h.recv() {
      Event::PeerMsg(msg) => peer_msgs.push(msg.data),
      Event::Custom(code) => {
        if code == lockstep_op_code {
          // write current step
          h.broadcast_all(Arc::new(
            (lockstepper.current_step().to_string() + ":\n").into_bytes(),
          ));

          // write all msg
          for data in &peer_msgs {
            h.broadcast_all(data.clone());
          }

          peer_msgs.clear();
          lockstepper.next_step();
        }
      }
      _ => break,
    }
  }
}
