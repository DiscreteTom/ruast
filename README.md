# Ruast

Event driven async messaging framework.

The rust version of [Rua](https://github.com/DiscreteTom/rua)!

## Installation

```toml
[dependencies]
rua = { git = "https://github.com/DiscreteTom/ruast" }
```

## Get Started

Run a websocket broadcast server:

```toml
# Cargo.toml
[package]
name = "ruast-test"
version = "0.1.0"
edition = "2018"

[dependencies]
tokio = { version = "1.12.0", features = ["full"] }
rua = { git = "https://github.com/DiscreteTom/ruast" }
rua-tungstenite = { git = "https://github.com/DiscreteTom/ruast" }
```

```rust
// main.rs
use rua::model::{Stoppable, Writable};
use rua::node::broadcast::StoppableBcNode;
use rua::node::Ctrlc;
use rua_tungstenite::listener::WsListener;

#[tokio::main]
pub async fn main() {
  // stoppable broadcaster
  // stop bc will stop all targets
  let bc = StoppableBcNode::default();

  // websocket listener at 127.0.0.1:8080
  WsListener::default()
    .on_new_peer({
      let mut bc = bc.clone();
      move |ws_node| {
        bc.add_target(ws_node.handle());
        ws_node
          .on_msg({
            let bc = bc.clone();
            move |data| bc.write(data).unwrap()
          })
          .spawn();
      }
    })
    .spawn()
    .await
    .expect("WebSocket listener failed to bind address");

  println!("WebSocket listener is running at ws://127.0.0.1:8080");

  // wait for ctrlc, stop all targets
  Ctrlc::new().on_signal(move || bc.stop()).wait().await;
}
```

## [More Examples](https://github.com/DiscreteTom/ruast/tree/main/rua/examples)
