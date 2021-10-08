use bytes::Bytes;
use rua::{
  controller::{EventHub, LockstepController},
  model::HubEvent,
  peer::StdioPeer,
};

#[tokio::main]
pub async fn main() {
  let mut peer_msgs = Vec::new();
  let mut h = EventHub::new(256);

  let lockstep_op_code = 0;
  let mut lockstepper = LockstepController::new(1000, h.tx_clone(), lockstep_op_code);
  lockstepper.next_step();

  h.add_peer(StdioPeer::new(0, h.tx_clone(), 256).build())
    .unwrap();

  loop {
    match h.recv().await {
      HubEvent::PeerMsg(msg) => peer_msgs.push(msg.data),
      HubEvent::Custom(code) => {
        if code == lockstep_op_code {
          // write current step
          h.broadcast_all(Bytes::from(
            (lockstepper.current_step().to_string() + ":\n").into_bytes(),
          ))
          .await;

          // write all msg
          for data in &peer_msgs {
            h.broadcast_all(data.clone()).await;
          }

          peer_msgs.clear();
          lockstepper.next_step();
        }
      }
      _ => break,
    }
  }
}
