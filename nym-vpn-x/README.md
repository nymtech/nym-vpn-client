# nymvpn-x

Ne**x**t desktop client application for NymVPN.

## Install

#### Prerequisites

- Rust
- Nodejs, latest LTS version recommended
- npm
- protobuf

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

#### Protobuf

Install `protobuf` from your system package manager or download it
from the repository releases
https://github.com/protocolbuffers/protobuf/releases and make sure
`protoc` is in your `PATH`

## Dev

To start the app in dev mode run:

```
RUST_LOG=info,nym_vpn_x=trace npm run dev:app
```

(tweak `RUST_LOG` env var as needed)

or via `cargo`

```
cd src-tauri
RUST_LOG=info,nym_vpn_x=trace cargo tauri dev
```

#### On Windows

In a PowerShell terminal run

```powershell
$env:RUST_LOG='debug,nym_vpn_x=trace'; cargo tauri dev; $env:RUST_LOG=$null
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

```shell
nymvpn-x --help
```

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
cd nym-vpn-x
npm i
mkdir dist

TAURI_PRIVATE_KEY=1234 TAURI_KEY_PASSWORD=1234 npm run tauri build
```

## Custom app config

The app looks for the config file `config.toml` under `nymvpn-x`
directory, full path is platform specific:

- Linux: `$XDG_CONFIG_HOME/nymvpn-x/` or `$HOME/.config/nymvpn-x/`
- macOS: `$HOME/Library/Application Support/nymvpn-x/`
- Windows: `C:\Users\<USER>\AppData\Roaming\nymvpn-x\`

For example on Linux the full path would be
`~/.config/nymvpn-x/config.toml`.

You can find the supported properties in the
[config schema](https://github.com/nymtech/nym-vpn-client/blob/main/nym-vpn-x/src-tauri/src/fs/config.rs)

**NOTE** The config file and all properties are optional
