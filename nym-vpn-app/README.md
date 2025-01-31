# nym-vpn-app

Desktop client application for [NymVPN](https://nym.com/), built with
[tauri](https://tauri.app/). Supports Linux and Windows.

For more information about NymVPN, its features, latest announcements,
Help Center, or to download the latest stable release, visit
[nym.com](https://nym.com/en).

## Installation

The following install methods are available:

### Arch Linux (AUR)

Install the following packages from the AUR:

```shell
yay -S nym-vpn-app nym-vpnd

# Also available as pre-built binaries
# yay -S nym-vpn-app-bin nym-vpnd-bin
```

Start the VPN service

```shell
sudo systemctl enable --now nym-vpnd.service
```

### Deb (Ubuntu, Debian)

Import the Nym repository, then install as usual

```shell
wget https://apt.nymtech.net/pool/main/n/nym-repo-setup/nym-repo-setup_1.0.1_amd64.deb -O /tmp/nym-repo-setup_1.0.1_amd64.deb
sudo dpkg -i /tmp/nym-repo-setup_1.0.1_amd64.deb

sudo apt install nym-vpn
```

### Flatpak (client app only)

The app is available on [Flathub](https://flathub.org/apps/net.nymtech.NymVPN)

```shell
flatpak install flathub net.nymtech.NymVPN
```

### AppImage (client app only)

The _AppImage_ is available in the
[releases](https://github.com/nymtech/nym-vpn-client/releases),
look for release tag `nym-vpn-app-v*` and download `NymVPN.AppImage`.

### Windows

The installer is available in the
[releases](https://github.com/nymtech/nym-vpn-client/releases).\
Look for release tag `nym-vpn-app-v*`, download
`NymVPN-setup.exe` and launch it.

---

## Development

#### Prerequisites

- Rust
- Nodejs, latest LTS version recommended
- npm
- protobuf

Some system libraries are required depending on the host platform.
Follow the instructions for your specific OS [here](https://tauri.app/start/prerequisites/)

#### Install

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
cargo install tauri-cli@^2.0.0-rc
cargo tauri help
```

#### Protobuf

Install `protobuf` from your system package manager or download it
from the repository releases
https://github.com/protocolbuffers/protobuf/releases and make sure
`protoc` is in your `PATH`

## Run dev

To start the app in dev mode run:

```
RUST_LOG=info,nym_vpn_app=trace npm run dev:app
```

(tweak `RUST_LOG` env var as needed)

or via `cargo`

```
cd src-tauri
RUST_LOG=info,nym_vpn_app=trace cargo tauri dev
```

#### On Windows

In a PowerShell terminal run

```powershell
$env:RUST_LOG='debug,nym_vpn_app=trace'; cargo tauri dev; $env:RUST_LOG=$null
```

## Dev in the browser

For convenience and better development experience, we can run the
app directly in the browser

```
npm run dev:browser
```

Then press `o` to open the app in the browser.

#### Tauri commands mock

Browser mode requires some of the tauri [commands](https://tauri.app/develop/calling-rust/#commands) (IPC calls) to be mocked.
When creating new tauri command, be sure to add the corresponding
mock definition into `src/dev/tauri-cmd-mocks/` and update
`src/dev/setup.ts` accordingly.

## CLI

We are using [clap](https://docs.rs/clap/latest/clap/) to handle CLI for the app.

```shell
nym-vpn-app --help
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
cd nym-vpn-app
npm i
mkdir dist

TAURI_SIGNING_PRIVATE_KEY=1234 TAURI_SIGNING_PRIVATE_KEY_PASSWORD=1234 npm run tauri build
```

## Custom app config

The app looks for the config file `config.toml` under `nym-vpn-app`
directory, full path is platform specific:

- Linux: `$XDG_CONFIG_HOME/nym-vpn-app/` or `$HOME/.config/nym-vpn-app/`
- macOS: `$HOME/Library/Application Support/nym-vpn-app/`
- Windows: `C:\Users\<USER>\AppData\Roaming\nym-vpn-app\`

For example on Linux the full path would be
`~/.config/nym-vpn-app/config.toml`.

You can find the supported properties in the
[config schema](https://github.com/nymtech/nym-vpn-client/blob/main/nym-vpn-app/src-tauri/src/fs/config.rs)

**NOTE** The config file and all properties are optional
