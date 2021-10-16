pub mod debug;
pub use debug::stdio::StdioPeerBuilder;
pub mod persistent;
pub use persistent::file::FilePeerBuilder;

#[macro_export]
macro_rules! basic_peer {
  () => {
    fn id(&self) -> u32 {
      self.id
    }
    fn set_tag(&mut self, tag: String) {
      self.tag = tag;
    }
    fn tag(&self) -> &str {
      &self.tag
    }
  };
}
