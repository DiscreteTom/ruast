use rua::model::{PeerEvent, Result};
use rua::peer::StdioPeerBuilder;

#[tokio::main]
pub async fn main() -> Result<()> {
  let stdio = {
    let mut builder = StdioPeerBuilder::new(16);
    let sink = builder.tx().clone();
    builder.id(0).sink(sink); // echo to self
    builder.build().await.unwrap()
  };

  tokio::signal::ctrl_c().await.unwrap();

  stdio.tx().send(PeerEvent::Stop).await.unwrap();

  Ok(())
}
