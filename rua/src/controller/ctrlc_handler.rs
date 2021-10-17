use tokio::{signal, sync::mpsc::Sender};

use crate::model::{HubEvent, Plugin};

use super::server::ServerManager;

pub struct CtrlcHandler {}

impl Plugin for CtrlcHandler {
  fn start(&self, _: u32, hub_tx: Sender<HubEvent>) {
    tokio::spawn(async move {
      signal::ctrl_c().await.expect("failed to listen for event");
      hub_tx.send(HubEvent::Custom(0)).await.unwrap();
    });
  }

  fn handle(&self, hub: &mut ServerManager) {}
}
