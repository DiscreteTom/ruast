use bytes::BytesMut;
use rua::lockstep::LockstepController;
use rua::model::{NodeEvent, Result};
use rua::node::StdioNode;
use tokio::sync::mpsc;

#[tokio::main]
pub async fn main() -> Result<()> {
  let (event_tx, mut event_rx) = mpsc::channel(16);
  let mut data = BytesMut::new();

  let stdio = {
    let mut builder = StdioNode::new(16);
    builder.id(0).sink(event_tx);
    builder.build().await.unwrap()
  };

  let (lockstep, mut step_rx) = LockstepController::new(1000);

  loop {
    tokio::select! {
      e = event_rx.recv() => {
        match e {
          None => break,
          Some(e) => {
            if let NodeEvent::Write(msg) = e {
              data.extend_from_slice(&msg[..]);
            }
          }
        }
      }
      step = step_rx.recv() => {
        match step {
          None => break,
          Some(step) => {
            let mut result = BytesMut::new();
            result.extend_from_slice(&(step.to_string()+":\n").into_bytes());
            result.extend_from_slice(&data.freeze());
            stdio.tx().send(NodeEvent::Write(result.freeze())).await.unwrap();
            data = BytesMut::new();
          }
        }
      }
      _ = tokio::signal::ctrl_c() => {
        break
      }
    }
  }

  stdio.tx().send(NodeEvent::Stop).await.unwrap();
  lockstep.stop().await;

  Ok(())
}
