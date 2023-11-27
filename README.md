
## Nym VPN CLI

A CLI VPN client that uses the Nym mixnet. It can optionally do the first connection to the entry gateway using wireguard, and it uses the Mullvad VPN wrapper around the wireguard-go userspace implementation.

### How to build

As a one-time command, from the `wireguard` directory, run the build script:

```
./build.sh
```

This will build inside the `build` directory the static library that wraps the wireguard-go implementation in FFI.

Next, to build the Rust CLI, run the following in the root of the project:

```
RUSTFLAGS='-L [PATH_TO_CLI_REPO]/build/lib/aarch64-apple-darwin' cargo build
```

replacing the `[PATH_TO_CLI_REPO]` with the absolute path to the `nym-vpn-cli` repository.


### How to run

The binary needs root permissions. The argument values have to be taken from a WireGuard configuration file.

```
$ ./target/debug/nym-vpn-cli --help
Usage: nym-vpn-cli [OPTIONS] --mixnet-client-path <MIXNET_CLIENT_PATH> --entry-gateway <ENTRY_GATEWAY> --exit-router <EXIT_ROUTER>

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
  -h, --help
          Print help
  -V, --version
          Print version
```

Example for a given configuration:
```
[Interface]
PrivateKey = AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
Address = 10.49.0.8 ,2001:db8:a160::8
DNS =  172.28.193.195, fd00::c:c1c3 

[Peer]
PublicKey = vhNLvkOBprXJDHnuhXz8wvxl8T8bxkia3xn5Ebk/8kI=
PresharedKey = BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=
AllowedIPs = 0.0.0.0/0,::/0
Endpoint = 185.19.30.168:51820
PersistentKeepalive = 55
```

you would run (TODO: update me! This is outdated):

```
 sudo ./target/debug/nym-vpn-cli --private-key "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=" --addresses 10.49.0.8 2001:db8:a160::8 --public-key "vhNLvkOBprXJDHnuhXz8wvxl8T8bxkia3xn5Ebk/8kI=" --allowed-ips 0.0.0.0/0 ::/0 --endpoint 185.19.30.168:51820 --psk "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=" --ipv4-gateway 172.28.193.195
```
