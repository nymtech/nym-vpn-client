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

## Required config

First you can provide a network configuration using en env file,
pick the relevant one [here](https://github.com/nymtech/nym/tree/develop/envs).
The mainnet config will be used by default if not provided.

Then create the main app config file `config.toml` under `nym-vpn`
directory, full path is platform specific:

- Linux: Resolves to `$XDG_CONFIG_HOME` or `$HOME/.config`
- macOS: Resolves to `$HOME/Library/Application Support`
- Windows: Resolves to `{FOLDERID_RoamingAppData}`

For example on Linux the path would be `~/.config/nym-vpn/config.toml`

```toml
# example config on Linux

# path to the env config file if you provide one
env_config_file = "/home/<USER>/.config/nym-vpn/qa.env"
```

## Dev

```
npm run dev:app
```

or via `cargo`

```
cd src-tauri
cargo tauri dev
```

**NOTE** Starting a VPN connection requires root privileges as it will set up a link interface.
If you want to connect during development, you need to run the app as root,
likely using `sudo` (or equivalent)

```shell
sudo -E RUST_LOG=debug cargo tauri dev
```

#### Logging

Rust logging (standard output) is controlled by the `RUST_LOG`
env variable

Example:

```
RUST_LOG=nym_vpn_ui=trace,nym_client_core=warn,nym_vpn_lib=info npm run dev:app
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
mock definition into `nym-vpn/ui/src/dev/tauri-cmd-mocks/` and
update `nym-vpn/ui/src/dev/setup.ts` accordingly.

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

```
npm run build:app
```
