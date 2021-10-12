use std::collections::HashMap;

use crate::{
  model::{HubEvent, Peer, PeerIdAllocator, Plugin, Result},
  peer::StdioPeerBuilder,
};

use super::{
  utils::{SimpleIdGenerator, SimplePeerIdAllocator},
  EventHub,
};

pub struct ServerManager {
  hub: EventHub,
  handle_ctrl_c: bool,
  use_stdio: bool,
  plugins: HashMap<u32, Box<dyn Plugin>>,
  plugin_id_allocator: SimpleIdGenerator,
  peer_id_allocator: Box<dyn PeerIdAllocator>,
}

impl ServerManager {
  pub fn new() -> Self {
    Self {
      hub: EventHub::new(256),
      handle_ctrl_c: true,
      use_stdio: false,
      plugins: HashMap::new(),
      plugin_id_allocator: SimpleIdGenerator::new(0),
      peer_id_allocator: Box::new(SimplePeerIdAllocator::new(0)),
    }
  }

  pub fn enable_stdio(&mut self, enable: bool) -> &Self {
    self.use_stdio = enable;
    self
  }

  pub fn peer_id_allocator(&mut self, allocator: Box<dyn PeerIdAllocator>) -> &Self {
    self.peer_id_allocator = allocator;
    self
  }

  pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) -> u32 {
    let id = self.plugin_id_allocator.next();
    self.plugins.insert(id, plugin);
    id
  }

  pub async fn add_peer(&self, peer: Box<dyn Peer>) -> Result<()> {
    self.hub.add_peer(peer)
  }

  pub async fn remove_peer(&self, id: u32) -> Result<()> {
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
