use std::{collections::HashMap, sync::Arc};

use tokio::sync::{
  mpsc::{self, Receiver, Sender},
  Mutex,
};

use crate::{
  model::{Peer, PeerBuilder, PeerIdAllocator, PeerMsg, Plugin, Result, ServerEvent},
  peer::StdioPeerBuilder,
};

use super::{
  utils::{SimpleIdGenerator, SimplePeerIdAllocator},
  PeerManager,
};

pub struct ServerManager {
  pm: Arc<Mutex<PeerManager>>,
  tx: Sender<ServerEvent>,
  rx: Receiver<ServerEvent>,
  handle_ctrl_c: bool,
  stdio: bool,
  stdio_id: u32,
  plugins: HashMap<u32, Box<dyn Plugin>>,
  plugin_id_allocator: SimpleIdGenerator,
  peer_id_allocator: Option<Box<dyn PeerIdAllocator>>,
  peer_msg_handler: Box<dyn Fn(PeerMsg, Arc<Mutex<PeerManager>>) + 'static>,
}

impl ServerManager {
  pub fn new(event_buffer: usize) -> Self {
    let (tx, rx) = mpsc::channel(event_buffer);
    let pm = Arc::new(Mutex::new(PeerManager::new()));

    Self {
      pm,
      tx,
      rx,
      handle_ctrl_c: true,
      stdio: false,
      stdio_id: 0,
      plugins: HashMap::new(),
      plugin_id_allocator: SimpleIdGenerator::new(0),
      peer_id_allocator: None,
      peer_msg_handler: Box::new(|_, _| {}),
    }
  }

  /// Enable StdioPeer with id=0.
  pub fn stdio(&mut self, enable: bool) -> &mut Self {
    self.stdio = enable;
    self
  }

  pub fn stdio_with_id(&mut self, id: u32) -> &mut Self {
    self.stdio = true;
    self.stdio_id = id;
    self
  }

  pub fn peer_id_allocator(&mut self, allocator: Box<dyn PeerIdAllocator>) -> &mut Self {
    self.peer_id_allocator = Some(allocator);
    self
  }

  /// Auto assign peer id from 1.
  pub fn auto_peer_id(&mut self, enable: bool) -> &mut Self {
    self.peer_id_allocator = if enable {
      Some(Box::new(SimplePeerIdAllocator::new(0)))
    } else {
      None
    };
    self
  }

  pub fn handle_ctrl_c(&mut self, enable: bool) -> &mut Self {
    self.handle_ctrl_c = enable;
    self
  }

  pub fn on_peer_msg(
    &mut self,
    f: impl Fn(PeerMsg, Arc<Mutex<PeerManager>>) + 'static,
  ) -> &mut Self {
    self.peer_msg_handler = Box::new(f);
    self
  }

  pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
    let id = self.plugin_id_allocator.next();
    self.plugins.insert(id, plugin);
  }

  pub async fn add_peer(&mut self, mut peer_builder: Box<dyn PeerBuilder>) -> Result<u32> {
    let id = match &mut self.peer_id_allocator {
      Some(allocator) => {
        let id = allocator.allocate(&peer_builder);
        peer_builder.id(id);
        id
      }
      _ => peer_builder
        .get_id()
        .ok_or("peer_builder must have an id when server.peer_id_allocator has not been set")?,
    };

    self
      .pm
      .lock()
      .await
      .add_peer(peer_builder.server_tx(self.tx.clone()).build().await?)?;
    Ok(id)
  }

  pub async fn remove_peer(&mut self, id: u32) -> Result<()> {
    self.pm.lock().await.remove_peer(id)
  }

  pub async fn start(&mut self) {
    // start plugins
    for (code, plugin) in &self.plugins {
      plugin.start(*code, self.tx.clone());
    }

    // stdio peer
    if self.stdio {
      self
        .add_peer({
          let mut pb = StdioPeerBuilder::new();
          pb.id(self.stdio_id);
          Box::new(pb)
        })
        .await
        .expect("can not build StdioPeer");
    }

    loop {
      match self.rx.recv().await.unwrap() {
        ServerEvent::PeerMsg(msg) => (self.peer_msg_handler)(msg, self.pm.clone()),
        ServerEvent::RemovePeer(id) => {
          // TODO: before remove peer
          self.remove_peer(id).await.unwrap();
          // TODO: failed to remove peer
          // TODO: after remove peer
        }
        ServerEvent::Custom(0) => {
          break;
        }
        ServerEvent::Custom(id) => {
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
