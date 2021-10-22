use bytes::BytesMut;
use rua::model::{NodeEvent, Result};
use rua::node::{Ctrlc, Lc, StdioNode};
use tokio::sync::mpsc;

#[tokio::main]
pub async fn main() -> Result<()> {
  // all node event will be send to the hub
  let (hub_tx, mut hub_rx) = mpsc::channel(16);

  // create a buffer to store data during a step
  let mut buffer = BytesMut::new();

  // create StdioNode, stdin => hub
  let stdio = StdioNode::new(16).sink(hub_tx.clone()).spawn();

  // create lockstep controller, step length = 1 second
  let (lc, mut step_rx) = Lc::new(1000);

  // handle ctrlc: send NodeEvent::Stop to the hub
  Ctrlc::new().sink(hub_tx).spawn().unwrap();

  loop {
    tokio::select! {
      e = hub_rx.recv() => {
        match e {
          None => break,
          Some(e) => {
            match e {
              NodeEvent::Write(data) => {
                // append data to buffer
                buffer.extend_from_slice(&data[..]);
              }
              NodeEvent::Stop => break, // handle ctrlc
            }
          }
        }
      }
      step = step_rx.recv() => {
        match step {
          None => break,
          Some(step) => {
            let mut result = BytesMut::new();
            // append current step number
            result.extend_from_slice(&(step.to_string() + ":\n").into_bytes());
            // append data during this step
            result.extend_from_slice(&buffer.freeze());

            stdio.send(NodeEvent::Write(result.freeze())).await.unwrap();
            buffer = BytesMut::new();
          }
        }
      }
    }
  }

  // stop StdioNode & lockstep controller
  stdio.send(NodeEvent::Stop).await.unwrap();
  lc.stop().await;

  Ok(())
}
