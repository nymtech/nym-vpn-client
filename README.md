# Nym VPN CLI

A commandline VPN client that uses the [Nym mixnet](https://nymtech.net). 

By default it will do 5-hops (incl entry and exit gateways).

```
                      ┌─►mix──┐  mix     mix
                      │       │
            entry     │       │                   exit
client ───► gateway ──┘  mix  │  mix  ┌─►mix ───► gateway ───► internet
                              │       │
                              │       │
                         mix  └─►mix──┘  mix
```

It can optionally do the first connection to the entry gateway using wireguard, and it uses Mullvad libraries for wrapping wireguard-go and to setup local routing rules to route all traffic to the TUN virtual network device.

## How to build

### Step 1.

Build the wireguard-go static library:

```sh
$ ./wireguard/build-wireguard.go.sh
```

This should create a local `build` directory with a static library `libwg.a` that wraps the wireguard-go implementation in a FFI.

```sh
$ ls build/lib/x86_64-unknown-linux-gnu/      
libwg.a  libwg.h
```

### Step 2.

Next, to build the Rust CLI, you need to add the library to the search path by prepending `cargo build` with `RUSTFLAGS`,

```sh
RUSTFLAGS='-L [PATH_TO_CLI_REPO]/build/lib/aarch64-apple-darwin' cargo build --release
```

replacing the `[PATH_TO_CLI_REPO]` with the absolute path to the `nym-vpn-cli` repository, and of course adjust the path to match the target arch.
Alternatively add rustflags to `.cargo/config.toml`

```
$ cat .cargo/config.toml 
[build]
rustflags = ['-L', '/home/nymuser/src/nym/nym-vpn-cli/build/lib/x86_64-unknown-linux-gnu']
```
and then run `cargo build --release` like normal.

## How to run

The binary needs root permissions to setup the TUN virtual network device.

### Case 1: connect to the entry gateway using a websocket connection.

```sh
$ sudo ./target/release/nym-vpn-cli --entry-gateway <ENTRY_GATEWAY> --exit-router <EXIT_ROUTER>
```

### Case 2: using WireGuard for the connection between the client and the entry gateway.

```sh
$ sudo ./target/release/nym-vpn-cli --entry-gateway <ENTRY_GATEWAY> --exit-router <EXIT_ROUTER> --enable-wireguard --private-key <PRIVATE_KEY>
```

The full set of flags are:

```
$ ./target/release/nym-vpn-cli --help
Usage: nym-vpn-cli [OPTIONS] --entry-gateway <ENTRY_GATEWAY> --exit-router <EXIT_ROUTER> --ip <IP>

Options:
  -c, --config-env-file <CONFIG_ENV_FILE>
          Path pointing to an env file describing the network
      --enable-wireguard
          Enable the wireguard traffic between the client and the entry gateway
      --mixnet-client-path <MIXNET_CLIENT_PATH>
          Path to the data directory of a previously initialised mixnet client, where the keys reside
      --entry-gateway <ENTRY_GATEWAY>
          Mixnet public ID of the entry gateway
      --exit-router <EXIT_ROUTER>
          Mixnet recipient address
      --private-key <PRIVATE_KEY>
          Associated private key
      --ip <IP>
          The IP address of the TUN device
      --disable-routing
          Disable routing all traffic through the VPN TUN device
  -h, --help
          Print help
  -V, --version
          Print version
```
