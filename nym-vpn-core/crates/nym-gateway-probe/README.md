# Nym Gateway Probe

Probe IPv4 and IPv6 interfaces of available gateways to check for the
set that passes a set of minumum service guarantees.


## Build

These instructions assume a debian based system. Adjust accordingly for your
preffered platform.

Install required dependencies
```sh
sudo apt install libdbus-1-dev libmnl-dev libnftnl-dev protobuf-compiler clang
```


Build piece by piece
```sh
cd nym-vpn-core/
# build the prober
cargo build -p nym-gateway-probe
```

You may need to adjust your `RUSTFLAGS` or `.cargo/config.toml` to ensure that
the golang wireguard library links properly.

## Usage

```sh
Usage: nym-gateway-probe [OPTIONS]

Options:
  -c, --config-env-file <CONFIG_ENV_FILE>
          Path pointing to an env file describing the network
  -g, --gateway <GATEWAY>
          The specific gateway specified by ID
  -n, --no-log
          Disable logging during probe
  -a, --amnezia-args <AMNEZIA_ARGS>
          Arguments to be appended to the wireguard config enabling amnezia-wg configuration
  -h, --help
          Print help
  -V, --version
          Print version
```

Examples

```sh
# Run a basic probe against the node with id "qj3GgGYg..."
nym-gateway-probe -g "qj3GgGYgGZZ3HkFrtD1GU9UJ5oNXME9eD2xtmPLqYYw"

# Run a basic probe against the node with id "qj3GgGYg..."
nym-gateway-probe -g "qj3GgGYgGZZ3HkFrtD1GU9UJ5oNXME9eD2xtmPLqYYw" -a "Jc=4\nJmin=40\mJmax=70\n"
```
