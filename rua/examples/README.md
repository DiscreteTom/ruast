# Examples

- `echo.rs` shows how to create a node, how to use a handle, how to use `cc!` to create an event handler, and how to use `Ctrlc` to stop your program.
  - stdin => stdout
- `file-persistent.rs` shows how to work with many handles.
  - stdin => file
- `tail-file.rs` shows how easy to write a `tail -f` application.
- `callback.rs` shows how to use callback functions to check whether a write is finished.
  - stdin => file => stdout (if error)
- `echo-lockstep.rs` shows how to interact with shared states, and how to realize lockstep output.
  - stdin => state
  - state => stdout (on step)
- `tcp-broadcaster.rs` shows how to use `TcpListener` and how to interact with `Broadcaster`, which will be very useful if you need to work with unknow number of nodes.
  - Use `nc localhost 8080` to connect to the tcp server.
