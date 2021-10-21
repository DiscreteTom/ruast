/// Implement `id()`, `sink()`, `tag()`, `tx()` for PeerBuilder.
///
/// # Examples
///
/// ```
/// pub struct MyPeerBuilder{};
///
/// impl MyPeerBuilder {
///   // implement id(), sink(), tag(), tx()
///   impl_peer_builder!(all);
///   
///   // or implement selected methods
///   // impl_peer_builder!(id, tag);
/// }
/// ```
#[macro_export]
macro_rules! impl_peer_builder {
  (sink) => {
    pub fn sink(&mut self, sink: Sender<PeerEvent>) -> &mut Self {
      self.sink = Some(sink);
      self
    }
  };

  (tx) => {
    pub fn tx(&self) -> &Sender<PeerEvent> {
      &self.tx
    }
  };

  (all) => {
    impl_peer_builder!(tx,sink);
  };

  ($($i:ident),+)=>{
    $(impl_peer_builder!($i);)*
  }
}
