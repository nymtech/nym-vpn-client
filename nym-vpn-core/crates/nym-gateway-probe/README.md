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
# from root of repo
make build-wireguard

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
  -c, --config-env-file <CONFIG_ENV_FILE>  Path pointing to an env file describing the network
  -g, --gateway <GATEWAY>
  -n, --no-log
  -h, --help                               Print help
  -V, --version                            Print version
```

