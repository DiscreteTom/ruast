use std::{collections::HashMap, sync::Arc};

use tokio::sync::{
  mpsc::{self, Receiver, Sender},
  Mutex,
};

use crate::{
  model::{HubEvent, Peer, PeerBuilder, PeerIdAllocator, PeerMsg, Plugin, Result},
  peer::StdioPeerBuilder,
};

use super::{
  utils::{SimpleIdGenerator, SimplePeerIdAllocator},
  EventHub,
};

pub struct ServerManager {
  hub: Arc<Mutex<EventHub>>,
  hub_tx: Sender<HubEvent>,
  hub_rx: Receiver<HubEvent>,
  handle_ctrl_c: bool,
  stdio: bool,
  plugins: HashMap<u32, Box<dyn Plugin>>,
  plugin_id_allocator: SimpleIdGenerator,
  peer_id_allocator: Box<dyn PeerIdAllocator>,
  peer_msg_handler: Box<dyn Fn(PeerMsg, Arc<Mutex<EventHub>>) + 'static>,
}

impl ServerManager {
  pub fn new(event_buffer: usize) -> Self {
    let (hub_tx, hub_rx) = mpsc::channel(event_buffer);
    let hub = Arc::new(Mutex::new(EventHub::with_tx(hub_tx.clone())));

    Self {
      hub,
      hub_tx,
      hub_rx,
      handle_ctrl_c: true,
      stdio: false,
      plugins: HashMap::new(),
      plugin_id_allocator: SimpleIdGenerator::new(0),
      peer_id_allocator: Box::new(SimplePeerIdAllocator::new(0)),
      peer_msg_handler: Box::new(|_, _| {}),
    }
  }

  pub fn stdio(&mut self, enable: bool) -> &mut Self {
    self.stdio = enable;
    self
  }

  pub fn peer_id_allocator(&mut self, allocator: Box<dyn PeerIdAllocator>) -> &mut Self {
    self.peer_id_allocator = allocator;
    self
  }

  pub fn handle_ctrl_c(&mut self, enable: bool) -> &mut Self {
    self.handle_ctrl_c = enable;
    self
  }

  pub fn on_peer_msg(&mut self, f: impl Fn(PeerMsg, Arc<Mutex<EventHub>>) + 'static) -> &mut Self {
    self.peer_msg_handler = Box::new(f);
    self
  }

  pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
    let id = self.plugin_id_allocator.next();
    self.plugins.insert(id, plugin);
  }

  pub async fn add_peer(&mut self, mut peer_builder: Box<dyn PeerBuilder>) -> Result<u32> {
    let id = self.peer_id_allocator.allocate(&peer_builder);
    self.hub.lock().await.add_peer(
      peer_builder
        .id(id)
        .hub_tx(self.hub_tx.clone())
        .build()
        .await?,
    )?;
    Ok(id)
  }

  pub async fn remove_peer(&mut self, id: u32) -> Result<()> {
    self.hub.lock().await.remove_peer(id)
  }

  pub async fn start(&mut self) {
    // start plugins
    for (code, plugin) in &self.plugins {
      plugin.start(*code, self.hub_tx.clone());
    }

    // stdio peer
    if self.stdio {
      self
        .add_peer(StdioPeerBuilder::new().boxed())
        .await
        .expect("can not build StdioPeer");
    }

    loop {
      match self.hub_rx.recv().await.unwrap() {
        HubEvent::PeerMsg(msg) => (self.peer_msg_handler)(msg, self.hub.clone()),
        HubEvent::RemovePeer(id) => {
          // TODO: before remove peer
          self.remove_peer(id).await.unwrap();
          // TODO: failed to remove peer
          // TODO: after remove peer
        }
        HubEvent::Custom(0) => {
          break;
        }
        HubEvent::Custom(id) => {
          if let Some(plugin) = self.plugins.get(&id) {
            // plugin.handle(self);
          } else {
            todo!()
          }
        }
      }
    }
  }
}
