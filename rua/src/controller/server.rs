use std::collections::HashMap;

use crate::{
  model::{HubEvent, Peer, Plugin, Result},
  peer::StdioPeerBuilder,
};

use super::EventHub;

pub struct ServerManager {
  hub: EventHub,
  handle_ctrl_c: bool,
  use_stdio: bool,
  plugins: HashMap<u32, Box<dyn Plugin>>,
  plugin_index: u32,
  peer_index: i32,
}

impl ServerManager {
  pub fn new() -> Self {
    Self {
      hub: EventHub::new(256),
      handle_ctrl_c: true,
      use_stdio: false,
      plugins: HashMap::new(),
      plugin_index: 0,
      peer_index: 0,
    }
  }

  pub fn enable_stdio(&mut self, enable: bool) -> &Self {
    self.use_stdio = enable;
    self
  }

  pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) -> u32 {
    self.plugin_index += 1;

    loop {
      if self.plugin_index == 0 {
        // overflow
        panic!("too many plugins")
      } else if self.plugins.contains_key(&self.plugin_index) {
        self.plugin_index += 1;
      } else {
        self.plugins.insert(self.plugin_index, plugin);
        return self.plugin_index;
      }
    }
  }

  pub async fn add_peer(&self, peer: Box<dyn Peer>) -> Result<()> {
    self.hub.add_peer(peer)
  }

  pub async fn remove_peer(&self, id: i32) -> Result<()> {
    self.hub.remove_peer(id).await
  }

  pub async fn start(&mut self) {
    // start plugins
    for (_, plugin) in &self.plugins {
      plugin.start();
    }

    // stdio peer
    if self.use_stdio {
      self
        .add_peer(
          StdioPeerBuilder::new()
            .id(0)
            .buffer(32)
            .hub_tx(self.hub.tx_clone())
            .build(),
        )
        .await
        .unwrap();
    }

    loop {
      match self.hub.recv().await {
        HubEvent::PeerMsg(msg) => todo!(),
        HubEvent::RemovePeer(id) => {
          // TODO: before remove peer
          self.remove_peer(id).await.unwrap();
          // TODO: failed to remove peer
          // TODO: after remove peer
        }
        HubEvent::Custom(id) => {
          if let Some(plugin) = self.plugins.get(&id) {
            plugin.handle(&self.hub);
          } else {
            todo!()
          }
        }
        HubEvent::Stop => break,
      }
    }
  }
}
