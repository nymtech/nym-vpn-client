
## Nym Wireguard CLI

A CLI that uses the Mullvad VPN wrapper around the wireguard-go userspace implementation.

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
Usage: nym-vpn-cli [OPTIONS] --private-key <PRIVATE_KEY> --public-key <PUBLIC_KEY> --endpoint <ENDPOINT> --ipv4-gateway <IPV4_GATEWAY>

Options:
      --private-key <PRIVATE_KEY>     Associated private key
      --addresses <ADDRESSES>...      Local IP addresses associated with a key pair
      --public-key <PUBLIC_KEY>       Peer's public key
      --allowed-ips <ALLOWED_IPS>...  Addresses that may be routed to the peer. Use `0.0.0.0/0` to route everything
      --endpoint <ENDPOINT>           IP address of the WireGuard server
      --psk <PSK>                     Preshared key (PSK)
      --ipv4-gateway <IPV4_GATEWAY>   IPv4 gateway
  -h, --help                          Print help
  -V, --version                       Print version

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

you would run:

```
 sudo ./target/debug/nym-vpn-cli --private-key "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=" --addresses 10.49.0.8 2001:db8:a160::8 --public-key "vhNLvkOBprXJDHnuhXz8wvxl8T8bxkia3xn5Ebk/8kI=" --allowed-ips 0.0.0.0/0 ::/0 --endpoint 185.19.30.168:51820 --psk "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=" --ipv4-gateway 172.28.193.195
```
