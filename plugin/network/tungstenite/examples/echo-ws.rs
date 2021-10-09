use std::{sync::mpsc, thread};

use rua::{controller::EventHub, model::Event, peer::StdioPeer};
use rua_tungstenite::listener::WebsocketListener;

fn main() {
  let mut current_peer_id = 0;

  let h = EventHub::new();
  h.add_peer(StdioPeer::new(current_peer_id, h.tx())).unwrap();
  current_peer_id += 1;

  let ws_listener_code = 0;
  let (peer_tx, peer_rx) = mpsc::channel();
  let ws_listener = WebsocketListener::new("127.0.0.1:8080", ws_listener_code, h.tx(), peer_tx);
  thread::spawn(move || ws_listener.start());

  loop {
    match h.recv() {
      Event::PeerMsg(msg) => {
        h.broadcast_all(msg.data);
      }
      Event::Custom(code) => {
        if code == ws_listener_code {
          h.add_peer(peer_rx.recv().unwrap().build(current_peer_id, h.tx()))
            .unwrap();
          current_peer_id += 1;
        }
      }
      _ => break,
    }
  }
}
