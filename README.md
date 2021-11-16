# Ruast

![GitHub](https://img.shields.io/github/license/DiscreteTom/ruast?style=flat-square)
![GitHub release (latest by date)](https://img.shields.io/github/v/release/DiscreteTom/ruast?style=flat-square)

Event driven async messaging framework.

The rust version of [Rua](https://github.com/DiscreteTom/rua)!

## Installation

```toml
[dependencies]
rua = { git = "https://github.com/DiscreteTom/ruast" }
```

## Getting Started

Run a websocket broadcast server:

```toml
# Cargo.toml
[package]
name = "test"
version = "0.1.0"
edition = "2018"

[dependencies]
tokio = { version = "1.12.0", features = ["full"] }
rua = { git = "https://github.com/DiscreteTom/ruast" }
rua-tungstenite = { git = "https://github.com/DiscreteTom/ruast" }
clonesure = "0.3.0"
```

```rust
// main.rs
use clonesure::cc;
use rua::node::{broadcast::Broadcaster, Ctrlc, StdioNode};
use rua_tungstenite::listener::WsListener;

/// Use `wscat -c ws://127.0.0.1:8080` to connect to the websocket server.
#[tokio::main]
pub async fn main() {
  // stoppable broadcaster
  let mut bc = Broadcaster::default();

  // websocket listener at 127.0.0.1:8080
  let ws = WsListener::bind("127.0.0.1:8080")
    .on_new_peer(cc!(|@mut bc, ws_node| {
      // new node will be added to the broadcaster
      bc.add_target(
        ws_node
          .on_msg(cc!(|@bc, data| bc.write(data)))
          .spawn()
      );
    }))
    .spawn()
    .await
    .expect("WebSocket listener failed to bind address");

  // also print to stdout
  bc.add_target(StdioNode::default().spawn());

  println!("WebSocket listener is running at ws://127.0.0.1:8080");

  // wait for ctrlc, stop all targets
  Ctrlc::default()
    .on_signal(move || {
      bc.stop_all();
      ws.stop()
    })
    .wait()
    .await
    .expect("failed to listen for ctrlc");
}
```

## [More Examples](https://github.com/DiscreteTom/ruast/tree/main/rua/examples)
