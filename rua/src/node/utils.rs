/// Implement `sink()`, `tx()` for PeerBuilder.
///
/// # Examples
///
/// ```
/// pub struct MyNode{};
///
/// impl MyNode {
///    impl_node!(sink, tx);
/// }
/// ```
#[macro_export]
macro_rules! impl_node {
  (sink) => {
    pub fn sink(mut self, sink: Tx) -> Self {
      self.sink = Some(sink);
      self
    }
  };

  (tx) => {
    pub fn tx(&self) -> &Tx {
      &self.tx
    }
  };

  ($($i:ident),+)=>{
    $(impl_node!($i);)*
  }
}
