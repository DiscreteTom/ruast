/// Implement `sink()`, `tx()` for PeerBuilder.
///
/// # Examples
///
/// ```
/// pub struct MyPeerBuilder{};
///
/// impl MyPeerBuilder {
///    impl_peer_builder!(sink, tx);
/// }
/// ```
#[macro_export]
macro_rules! impl_peer_builder {
  (sink) => {
    pub fn sink(mut self, sink: Sender<NodeEvent>) -> Self {
      self.sink = Some(sink);
      self
    }
  };

  (tx) => {
    pub fn tx(&self) -> &Sender<NodeEvent> {
      &self.tx
    }
  };

  ($($i:ident),+)=>{
    $(impl_peer_builder!($i);)*
  }
}
