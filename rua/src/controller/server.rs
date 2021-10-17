use std::collections::HashMap;

use crate::{
  model::{HubEvent, Peer, PeerBuilder, PeerIdAllocator, Plugin, Result},
  peer::StdioPeerBuilder,
};

use super::{
  utils::{SimpleIdGenerator, SimplePeerIdAllocator},
  EventHub,
};

pub struct ServerManager {
  hub: EventHub,
  handle_ctrl_c: bool,
  stdio: bool,
  plugins: HashMap<u32, Box<dyn Plugin>>,
  plugin_id_allocator: SimpleIdGenerator,
  peer_id_allocator: Box<dyn PeerIdAllocator>,
}

impl ServerManager {
  pub fn new(event_buffer: usize) -> Self {
    Self {
      hub: EventHub::new(event_buffer),
      handle_ctrl_c: true,
      stdio: false,
      plugins: HashMap::new(),
      plugin_id_allocator: SimpleIdGenerator::new(0),
      peer_id_allocator: Box::new(SimplePeerIdAllocator::new(0)),
    }
  }

  pub fn stdio(&mut self, enable: bool) -> &Self {
    self.stdio = enable;
    self
  }

  pub fn peer_id_allocator(&mut self, allocator: Box<dyn PeerIdAllocator>) -> &Self {
    self.peer_id_allocator = allocator;
    self
  }

  pub fn handle_ctrl_c(&mut self, enable: bool) -> &Self {
    self.handle_ctrl_c = enable;
    self
  }

  pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
    let id = self.plugin_id_allocator.next();
    self.plugins.insert(id, plugin);
  }

  pub async fn add_peer(&mut self, mut peer_builder: Box<dyn PeerBuilder>) -> Result<u32> {
    let id = self.peer_id_allocator.allocate(&peer_builder);
    self.hub.add_peer(
      peer_builder
        .id(id)
        .hub_tx(self.hub.tx.clone())
        .build()
        .await?,
    )?;
    Ok(id)
  }

  pub fn remove_peer(&mut self, id: u32) -> Result<()> {
    self.hub.remove_peer(id)
  }

  pub async fn start(&mut self) {
    // start plugins
    for (code, plugin) in &self.plugins {
      plugin.start(*code);
    }

    // stdio peer
    if self.stdio {
      self
        .add_peer(StdioPeerBuilder::new().boxed())
        .await
        .expect("can not build StdioPeer");
    }

    loop {
      match self.hub.recv().await {
        HubEvent::PeerMsg(msg) => todo!(),
        HubEvent::RemovePeer(id) => {
          // TODO: before remove peer
          self.remove_peer(id).unwrap();
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
      }
    }
  }
}
