# nymvpn-x

Ne**x**t desktop client application for NymVPN.

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

## Dev

To start the app in dev mode run:

```
RUST_LOG=info,nymvpn_x=trace npm run dev:app
```

(tweak `RUST_LOG` env var as needed)

or via `cargo`

```
cd src-tauri
RUST_LOG=info,nymvpn_x=trace cargo tauri dev
```

#### Disabling the splash-screen

While developing, you might want to disable the splash-screen
to speedup app loading time.
Either set the `APP_NOSPLASH` env variable to `true` or pass the
`--nosplash` flag to the app

```shell
npm run dev:app -- -- -- --nosplash
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

## CLI

We are using [clap](https://docs.rs/clap/latest/clap/) to handle CLI for the app.

In dev mode, you can pass CLI arguments and flags with the `--` separator

```shell
npm run dev:app -- -- -- --help
# or
cargo tauri dev -- -- --help
```

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

To build run the following commands

```
cd nymvpn-x
npm i
mkdir dist

TAURI_PRIVATE_KEY=1234 TAURI_KEY_PASSWORD=1234 npm run tauri build
```

## Custom app config

The app looks for the config file `config.toml` under `nym-vpn`
directory, full path is platform specific:

- Linux: `$XDG_CONFIG_HOME/nym-vpn/` or `$HOME/.config/nym-vpn/`
- macOS: `$HOME/Library/Application Support/nym-vpn/`
- Windows: `C:\Users\<USER>\AppData\Roaming\nym-vpn\`

For example on Linux the full path would be
`~/.config/nym-vpn/config.toml`.

**NOTE** All properties are optional

```toml
# absolute path to a custom network configuration file
env_config_file = "/home/<USER>/.config/nym-vpn/custom.env"
# Address of NymVpn daemon to connect to (gRPC server endpoint)
daemon_address = "http://localhost:1234"
# IP address of the DNS server to use when connected to the VPN
dns_server = "1.1.1.1"
```
