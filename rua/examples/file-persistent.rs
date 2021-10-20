use rua::model::{PeerEvent, Result};
use rua::peer::persistent::file::FilePeerBuilder;
use rua::{broadcaster::Broadcaster, peer::StdioPeerBuilder};

#[tokio::main]
pub async fn main() -> Result<()> {
  let mut bc = Broadcaster::new(16);

  bc.add_target({
    let mut builder = StdioPeerBuilder::new(16);
    builder.id(0).sink(bc.tx().clone());
    builder.build().await.unwrap()
  })
  .await;

  bc.add_target({
    let mut builder = FilePeerBuilder::new(16);
    builder.id(1).filename("log.txt".to_string());
    builder.build().await.unwrap()
  })
  .await;

  tokio::signal::ctrl_c().await.unwrap();

  bc.tx().send(PeerEvent::Stop).await.ok();
  bc.stop().await;

  Ok(())
}
