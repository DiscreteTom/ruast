use std::{error::Error, io, sync::{Arc, mpsc::Sender}};

use crate::model::{Peer, PeerMsg, ServerEvent};

pub struct StdioPeer<'a>{
  msg_sender: Option<Sender<PeerMsg<'a>>>,
  tag: String,
  id: Option<i32>,
}

impl<'a> StdioPeer<'a>{
  fn new()->StdioPeer<'a>{
    StdioPeer{
      msg_sender: None,
      tag: String::from("stdio"),
      id: None
    }
  }
}

impl<'a> Peer<'a> for StdioPeer<'a>{
  fn write(&self, data: &Arc<&[u8]>) -> Result<(), Box<dyn Error>>{
    println!("{}", String::from_utf8_lossy(data));
    Ok(())
  }
  fn close(&self) -> Result<(), Box<dyn Error>>{
    Ok(())
  }
  fn start(&self){
    let mut line = String::new();
    io::stdin().read_line(&mut line)
        .expect("Failed to read line");
  }
  fn activate(&self, id: i32, msg_sender: Sender<ServerEvent>){}
  fn id(&self) -> i32{
    self.id.unwrap()
  }
  fn set_tag(&self, tag: &str){}
  fn tag(&'a self) -> &'a str{
    &self.tag
  }
}