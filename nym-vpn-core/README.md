# Nym VPN Core

## Build

These instructions assume a debian based system. Adjust accordingly for your
preferred platform.

Install required dependencies
```sh
sudo apt install libdbus-1-dev libmnl-dev libnftnl-dev protobuf-compiler
```


Build the wireguard library

```sh
# from the root of the repository
make build-wireguard
```

Build VPN libraries and executables

```sh
cd nym-vpn-core/

# build only the the vpn daemon
cargo build -p nym-vpnd

# build all 
cargo build --release
```

You may need to adjust your `RUSTFLAGS` or `.cargo/config.toml` to ensure that
the golang wireguard library links properly.

## Build for Windows from MacOS

Install toolchain
```sh
brew install mingw-w64
rustup target add x86_64-pc-windows-gnu
```

Configure linker in .cargo/config.toml:
```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
```

Then
```sh
cargo build --target=x86_64-pc-windows-gnu -p nym-vpn-lib
```