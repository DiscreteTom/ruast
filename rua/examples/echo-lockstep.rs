use bytes::Bytes;
use rua::{
  controller::{EventHub, LockstepController},
  model::{HubEvent, PeerBuilder, Result},
  peer::StdioPeerBuilder,
};
use tokio::sync::mpsc;

#[tokio::main]
pub async fn main() -> Result<()> {
  let mut peer_msgs = Vec::new();
  let mut h = EventHub::new();
  let (tx, mut rx) = mpsc::channel(256);

  let lockstep_op_code = 0;
  let mut lockstepper = LockstepController::new(1000, tx.clone(), lockstep_op_code);
  lockstepper.next_step();

  h.add_peer(
    StdioPeerBuilder::new()
      .id(0)
      .hub_tx(tx.clone())
      .build()
      .await?,
  )?;

  loop {
    match rx.recv().await.unwrap() {
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
      HubEvent::RemovePeer(id) => h.remove_peer(id)?,
      _ => break,
    }
  }

  Ok(())
}
