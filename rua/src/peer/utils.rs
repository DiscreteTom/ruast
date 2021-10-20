/// Easily implement the Peer trait.
///
/// # Examples
///
/// ```
/// use async_trait::async_trait;
///
/// pub struct MyPeer{};
///
/// #[async_trait]
/// impl Peer for MyPeer {
///   // implement id(), set_tag(), tag()
///   impl_peer!(all);
///   
///   // or implement selected methods
///   // impl_peer!(id, tag);
///
///   // there are still some methods you need to implement manually.
///   async fn write(&mut self, data: Bytes) -> Result<()> { Ok(()) }
///   fn stop(&mut self){}
/// }
/// ```
// #[macro_export]
// macro_rules! impl_peer {
//   (id) => {
//     fn id(&self) -> u32 {
//       self.id
//     }
//   };
//   (set_tag) => {
//     fn set_tag(&mut self, tag: String) {
//       self.tag = tag;
//     }
//   };
//   (tag) => {
//     fn tag(&self) -> &str {
//       &self.tag
//     }
//   };
//   (all) => {
//     impl_peer!(id, set_tag, tag);
//   };
//   ($($i:ident),+) =>{
//     $(impl_peer!($i);)*
//   };
// }

/// Easily implement the PeerBuilder trait.
///
/// # Examples
///
/// ```
/// use async_trait::async_trait;
///
/// pub struct MyPeerBuilder{};
///
/// #[async_trait]
/// impl PeerBuilder for MyPeerBuilder {
///   // implement id(), server_tx(), tag(), get_id(), get_tag()
///   impl_peer_builder!(all);
///   
///   // or implement selected methods
///   // impl_peer_builder!(id, tag);
///
///   // there are still some methods you need to implement manually.
///   async fn build(&mut self) -> Result<Box<dyn Peer>> { Ok(...) }
/// }
/// ```
// #[macro_export]
// macro_rules! impl_peer_builder {
//   (id) => {
//     fn id(&mut self, id: u32) -> &mut dyn PeerBuilder {
//       self.id = Some(id);
//       self
//     }
//   };

//   (server_tx) => {
//     fn server_tx(&mut self, tx: Sender<ServerEvent>) -> &mut dyn PeerBuilder {
//       self.server_tx = Some(tx);
//       self
//     }
//   };

//   (tag) => {
//     fn tag(&mut self, tag: String) -> &mut dyn PeerBuilder {
//       self.tag = Some(tag);
//       self
//     }
//   };
//   (get_id) => {
//     fn get_id(&self) -> Option<u32> {
//       self.id
//     }
//   };
//   (get_tag) => {
//     fn get_tag(&self) -> &str {
//       &self.tag.as_ref().unwrap()
//     }
//   };
//   (all) => {
//     impl_peer_builder!(id,server_tx,tag,get_id,get_tag);
//   };
//   ($($i:ident),+)=>{
//     $(impl_peer_builder!($i);)*
//   }
// }

#[macro_export]
macro_rules! impl_peer_builder {
  (id) => {
    pub fn id(&mut self, id: u32) -> &mut Self {
      self.id = Some(id);
      self
    }
  };

  (sink) => {
    pub fn sink(&mut self, sink: Sender<PeerEvent>) -> &mut Self {
      self.sink = Some(sink);
      self
    }
  };

  (tag) => {
    pub fn tag(&mut self, tag: String) -> &mut Self {
      self.tag = tag;
      self
    }
  };
  (tx) => {
    pub fn tx(&self) -> &Sender<PeerEvent> {
      &self.tx
    }
  };
  (all) => {
    impl_peer_builder!(id,tx,tag,sink);
  };
  ($($i:ident),+)=>{
    $(impl_peer_builder!($i);)*
  }
}
