The `forward_messages` function at test/test-rpc/src/transport.rs:241 is a message multiplexer that routes different types of data between a serial connection and multiple channels.

It takes messages from three sources and forwards them appropriately:

1. From serial stream: Deserializes incoming frames and routes them:
   - Frame::TestRunner → forwards to the tarpc runner channel
   - Frame::DaemonRpc → forwards to the daemon gRPC forwarder
   - Frame::Handshake → signals connection establishment
2. From runner channel: Serializes outgoing tarpc messages as Frame::TestRunner and sends over serial
3. From daemon forwarder: Forwards gRPC data as Frame::DaemonRpc frames over serial
4. Handshake messages: Sends periodic handshake frames to maintain connection state

The function uses `futures::select!` to handle all these concurrent message flows until any channel closes, then returns.

The forward_messages function is the core communication bridge in this VM-based testing architecture:

# Message Flow Context:

Host (test-manager) <-> VM (test-runner) via serial port

## On the Host Side (test-manager):

- Creates client transports via create_client_transports() at `test/test-manager/src/run_tests.rs:117-120`
- Opens serial connection to VM's PTY: `tokio_serial::SerialStream::open(pty_path)`
- Gets two channels:
  - runner_transport -> for tarpc test RPC calls
  - mullvad_daemon_transport -> for gRPC calls to Mullvad daemon inside VM

## On the VM Side (test-runner):

- Creates server transports via create_server_transports() at test/test-runner/src/main.rs:612-615
- Opens serial connection to host: tokio_serial::SerialStream::open(path)
- Spawns forward_to_mullvad_daemon_interface() to bridge gRPC calls to local Mullvad daemon

The `forward_messages` Function Routes:

1. Test RPC: Host test commands ↔ VM test-runner service (tarpc protocol)
2. Daemon gRPC: Host test client ↔ VM Mullvad daemon (raw gRPC over socket)
3. Handshakes: Connection establishment/keep-alive

Once the VPN activates inside the VM, network connectivity gets redirected through VPN tunnels, potentially breaking SSH/network-based communication. The serial port provides a reliable out-of-band channel that bypasses the network stack entirely.

# Single Serial Connection with Frame Multiplexing

There's actually one physical serial connection that carries both types of traffic, multiplexed by frame type:

## Host Side (test-manager):

```rust
// One serial stream
let serial_stream = tokio_serial::SerialStream::open(pty_path);

// Creates TWO logical channels from the same physical stream:
let (runner_transport, mullvad_daemon_transport, ...) =
    create_client_transports(serial_stream);
```

## VM Side (test-runner):

```rust
// Same single serial stream
let serial_stream = tokio_serial::SerialStream::open(path);

// Creates TWO logical channels from the same physical stream:
let (runner_transport, mullvad_daemon_transport, ...) =
    create_server_transports(serial_stream);
```

The forward_messages Multiplexing:

`Frame::TestRunner` → Routes to/from runner_transport (tarpc test RPC)
`Frame::DaemonRpc` → Routes to/from mullvad_daemon_transport (gRPC proxy)

Inside VM:

- mullvad_daemon_transport feeds into forward_to_mullvad_daemon_interface()
- This function bridges the Frame::DaemonRpc messages from the serial port to the local Mullvad daemon's Unix socket/named pipe

So the flow is:

Host gRPC client → Frame::DaemonRpc over serial → VM forward_to_mullvad_daemon_interface() → Local Mullvad daemon socket

The forward_messages function is the key multiplexer that splits the single serial stream into these two logical channels based on frame type.
