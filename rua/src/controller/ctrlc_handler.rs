use tokio::{signal, sync::mpsc::Sender};

use crate::model::{Plugin, ServerEvent};

use super::server::ServerManager;

pub struct CtrlcHandler {}

impl Plugin for CtrlcHandler {
  fn start(&self, _: u32, server_tx: Sender<ServerEvent>) {
    tokio::spawn(async move {
      signal::ctrl_c().await.expect("failed to listen for event");
      server_tx.send(ServerEvent::Custom(0)).await.unwrap();
    });
  }

  fn handle(&self, hub: &mut ServerManager) {}
}
