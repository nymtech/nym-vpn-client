# Rust Wireguard-go Wrapper

This library wraps `wireguard-go` making it avaiable for transparent use to other
rust crates.

## Usage

Using this crate requires building the `libwg.a` library file and `libwg.h` header
file. 

```toml
nym-wg-go.workspace = true
```

```sh
# In the root of the repo
make build-wireguard

# build this library (or downstream projects)
cargo build -p nym-wg-go
```

## Amnezia

To use the AmneziaWG version we need to enable the `amnezia` feature in this crate.

```toml
nym-wg-go = { workspace=true, features=["amnezia"]}
```

There are several helpers that build amnezia configurations for you so you do not have to instantiate the 

```rs
/// Disabled Amnezia Configuration, i.e. run as normal wireguard.
let _ = AmneziaConfig::OFF;

/// Enables only the minimum Amnezia features, while ensuring compatibility with plain
/// wireguard peers.
let _ = AmneziaConfig::BASE;

/// Creates a randomized configuration with parameters within suggested ranges.
let _ = AmneziaConfig::rand(rand::thread_rng());
```

If you would like full control over the configuration used by Amnezia you can construct an `AmneziaConfig` object.


```rs
// src/amnezia.rs
// Accessible through `Config > InterfaceConfig > AmneziaConfig`

pub struct AmneziaConfig {
    /// Jc - Count of junk packets to send BEFORE sending the handshake Init message.
    pub junk_pkt_count: u8, // jc
    /// Jmin - Minimum size in bytes of the Junk packets enabled by `junk_pkt_count`
    pub junk_pkt_min_size: u16, // jmin
    /// Jmax - Maximum size in bytes of the Junk packets enabled by `junk_pkt_count`
    pub junk_pkt_max_size: u16, // jmax
    /// S1 - Numer of byte to PREPEND to the Handshake init message
    pub init_pkt_junk_size: u16, // s1
    /// S2 - Number of bytes to PREPEND to the Handshake response message
    pub response_pkt_junk_size: u16, // s2
    /// H1 - Re-map handshake Init packet header type indicator to this value
    pub init_pkt_magic_header: i32, // h1
    /// H2 - Re-map handshake reponse packet header type indicator to this value
    pub response_pkt_magic_header: i32, // h2
    /// H3 - Re-map under load packet header type indicator to this value
    pub under_load_pkt_magic_header: i32, // h3
    /// H4 - Re-map transport packet header type indicator to this value
    pub transport_pkt_magic_header: i32, // h4
}
```
