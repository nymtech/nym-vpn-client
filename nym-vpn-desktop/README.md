# NymVPN desktop client app

Desktop client application of NymVPN.

## Install

#### Prerequisites

- Rust
- Nodejs, latest LTS version recommended
- npm

Some system libraries are required depending on the host platform.
Follow the instructions for your specific OS [here](https://tauri.app/v1/guides/getting-started/prerequisites)

#### To install run

```
npm i
```

#### Optional

Tauri CLI (`tauri-cli`) is provided as a local project package. To
run tauri commands you can use

```
npm run tauri help
```

If you want to run tauri through cargo you can install it on your
system, then you can run tauri commands via `cargo`

```
cargo install tauri-cli
cargo tauri help
```

## App config

The app looks for the config file `config.toml` under `nym-vpn`
directory, full path is platform specific:

- Linux: `$XDG_CONFIG_HOME/nym-vpn/` or `$HOME/.config/nym-vpn/`
- macOS: `$HOME/Library/Application Support/nym-vpn/`
- Windows: `C:\Users\<USER>\AppData\Roaming\nym-vpn\`

For example on Linux the full path would be
`~/.config/nym-vpn/config.toml`.

```toml
env_config_file = "/home/<USER>/.config/nym-vpn/sandbox.env"
default_entry_node_location_code = "FR"
default_exit_node_location_code = "DE"
```

`env_config_file` is the absolute path to a network configuration
file, pick the relevant one
[here](https://github.com/nymtech/nym/tree/develop/envs).

**NOTE** The sandbox config will be used by default if no config is provided.

`default_entry_node_location_code` and `default_exit_node_location_code` are the
default country codes for the entry and exit nodes respectively.
Available location codes can be found
[here](nym-vpn-desktop/src-tauri/src/country.rs).

## Dev

#### build wireguard-go (required)

From the repo root run

```
./wireguard/build-wireguard-go.sh
```

Then you need to provide the lib path to the rust library search
path. Create the file `.cargo/config.toml` from repo root

```config.toml
rustflags = ['-L', '/<ABSOLUTE_PATH_TO>/nym-vpn-client/build/lib/<PLATFORM_ARCH>']
```

or provide the env variable in each commands

```
RUSTFLAGS="-L /<ABSOLUTE_PATH_TO>/nym-vpn-client/build/lib/<PLATFORM_ARCH>" npm run dev:app
```

Replace `<ABSOLUTE_PATH_TO>` accordingly.
Replace `<PLATFORM_ARCH>` by your host specific platform and arch:

- Linux: `x86_64-unknown-linux-gnu`
- Mac x86_64: `x86_64-apple-darwin`
- Mac M1/M2/M3: `aarch64-apple-darwin`
- Windows: `x86_64-pc-windows-msvc`

To start the app in dev mode run:

```
npm run dev:app
```

or via `cargo`

```
cd src-tauri
cargo tauri dev
```

**NOTE** Starting a VPN connection requires root privileges as it
will set up a link interface.
If you want to connect during development, you need to run the app
as root, likely using `sudo` (or equivalent)

```shell
sudo -E RUST_LOG=debug cargo tauri dev
```

#### Logging

Rust logging (standard output) is controlled by the `RUST_LOG`
env variable

Example:

```
RUST_LOG=nym_vpn_desktop=trace,nym_client_core=warn,nym_vpn_lib=info npm run dev:app
```

or

```
cd src-tauri
RUST_LOG=trace cargo tauri dev
```

## Dev in the browser

For convenience and better development experience, we can run the
app directly in the browser

```
npm run dev:browser
```

Then press `o` to open the app in the browser.

#### Tauri commands mock

Browser mode requires all tauri [commands](https://tauri.app/v1/guides/features/command) (IPC calls) to be mocked.
When creating new tauri command, be sure to add the corresponding
mock definition into `src/dev/tauri-cmd-mocks/` and update
`src/dev/setup.ts` accordingly.

## Type bindings

[ts-rs](https://github.com/Aleph-Alpha/ts-rs) can be used to generate
TS type definitions from Rust types

To generate bindings, first
[annotate](https://github.com/Aleph-Alpha/ts-rs/blob/main/example/src/lib.rs)
Rust types, then run

```
cd src-tauri
cargo test
```

Generated TS types will be located in `src-tauri/bindings/`

## Build

First build `wireguard-go` lib and set `RUSTFLAGS` accordingly,
see [here](#build-wireguard-go-required)

Then run the following commands

```
cd nym-vpn-desktop
npm i
mkdir dist

TAURI_PRIVATE_KEY=1234 TAURI_KEY_PASSWORD=1234 npm run tauri build
```
