use bytes::Bytes;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
pub enum PeerEvent {
  Write(Bytes),
  Stop,
}

// #[async_trait]
// pub trait PeerBuilder {
//   fn id(&mut self, id: u32) -> &mut dyn PeerBuilder;
//   fn tag(&mut self, tag: String) -> &mut dyn PeerBuilder;
//   fn server_tx(&mut self, server_tx: Sender<ServerEvent>) -> &mut dyn PeerBuilder;
//   async fn build(&mut self) -> Result<Box<dyn Peer + Send>>;

//   fn get_id(&self) -> Option<u32>;
//   fn get_tag(&self) -> &str;
// }

// #[derive(Debug)]
// pub enum ServerEvent {
//   Custom(u32),
//   PeerMsg(PeerMsg),
//   RemovePeer(u32),
// }

// #[derive(Debug)]
// pub enum Error {
//   PeerNotExist(u32),
//   PeerAlreadyExist(u32),
// }

// impl fmt::Display for Error {
//   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//     match self {
//       Error::PeerNotExist(id) => write!(f, "peer not exist, id={}", id),
//       Error::PeerAlreadyExist(id) => write!(f, "peer already exist, id={}", id),
//     }
//   }
// }

// impl std::error::Error for Error {}
