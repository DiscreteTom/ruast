pub mod debug;
pub use debug::stdio::StdioPeerBuilder;
pub mod persistent;
pub use persistent::file::FilePeerBuilder;

#[macro_export]
macro_rules! impl_peer {
  (id) => {
    fn id(&self) -> u32 {
      self.id
    }
  };
  (set_tag) => {
    fn set_tag(&mut self, tag: String) {
      self.tag = tag;
    }
  };
  (tag) => {
    fn tag(&self) -> &str {
      &self.tag
    }
  };
  (all) => {
    impl_peer!(id);
    impl_peer!(set_tag);
    impl_peer!(tag);
  };
}
