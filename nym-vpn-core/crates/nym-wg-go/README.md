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

### Amnezia

To use the AmneziaWG version we need to enable the `amnezia` feature
in this crate.

```toml
nym-wg-go = { workspace=true, features=["amnezia"]}
```

The `AmneziaConfig` ( `Config > InterfaceConfig > AmneziaConfig`) can then be used
to enable and configure amnezia.

```rs
pub struct AmneziaConfig {
    /// Jc - Count of junk packets to send BEFORE sending the handshake Init message.
    pub junk_pkt_count: u8, // Jc
    /// Jmin - Minimum size in bytes of the Junk packets enabled by `junk_pkt_count`
    pub junk_pkt_min_size: u16, // Jmin
    /// Jmax - Maximum size in bytes of the Junk packets enabled by `junk_pkt_count`
    pub junk_pkt_max_size: u16, // Jmax
    /// S1 - Numer of byte to PREPEND to the Handshake init message
    pub init_pkt_junk_size: u16, // S1
    /// S2 - Number of bytes to PREPEND to the Handshake response message
    pub response_pkt_junk_size: u16, // S2
    /// H1 - Re-map handshake Init packet header type indicator to this value
    pub init_pkt_magic_header: i32, // H1
    /// H2 - Re-map handshake reponse packet header type indicator to this value
    pub response_pkt_magic_header: i32, // H2
    /// H3 - Re-map under load packet header type indicator to this value
    pub under_load_pkt_magic_header: i32, // H3
    /// H4 - Re-map transport packet header type indicator to this value
    pub transport_pkt_magic_header: i32, // H4
}
```
