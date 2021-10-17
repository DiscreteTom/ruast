use rua::{controller::server::ServerManager, model::Result};

#[tokio::main]
pub async fn main() -> Result<()> {
  // create a server
  let mut s = ServerManager::new(256);

  // enable stdio
  s.stdio(true);

  // a customizable way to enable stdio
  // s.add_peer(
  //   StdioPeerBuilder::new()
  //     .output_selector(|data| !data.starts_with(b"#"))
  //     .boxed(),
  // )
  // .await?;

  s.on_peer_msg(|msg, hub| {
    tokio::spawn(async move { hub.lock().await.echo(msg).await.unwrap() });
  });

  s.start().await;

  Ok(())
}
