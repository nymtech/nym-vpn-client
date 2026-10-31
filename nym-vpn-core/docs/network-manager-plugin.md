# Building nym-nm-vpn-plugin

The plugin lets NetworkManager start, stop and show the NymVPN tunnel run by `nym-vpnd`. It is
Linux only and ships as the `nym-vpn-networkmanager` Debian package.

It is made of two parts that are built separately:

| Part | Source | Build | Output |
| --- | --- | --- | --- |
| Plugin service `nm-nymvpn-service` | `src/` (Rust) | `cargo build` | `target/debug/` or `target/release/` |
| Editor libraries | `editor/` (C) | `make` | `editor/build/*.so` |

Source paths are relative to `crates/nym-nm-vpn-plugin/`.

The service is the process NetworkManager starts to connect and disconnect the VPN. It forwards
those requests to `nym-vpnd`, which keeps doing all routing, firewall and DNS work.

The editor libraries describe the VPN type to NetworkManager clients (`nmcli`, GNOME Settings,
nm-connection-editor):

- `libnm-vpn-plugin-nymvpn.so`: GTK-free core plugin, loaded by `nmcli` and other libnm clients.
- `libnm-vpn-plugin-nymvpn-editor.so`: editor page for GTK 3 (nm-connection-editor, nm-applet).
- `libnm-gtk4-vpn-plugin-nymvpn-editor.so`: editor page for GTK 4 (GNOME Settings).

`cargo build` does not build the C code, so `make` has to run before packaging.

## Prerequisites

On Debian or Ubuntu:

```sh
sudo apt-get install build-essential pkg-config protobuf-compiler \
    libdbus-1-dev libmnl-dev libnftnl-dev \
    libnm-dev libgtk-3-dev libgtk-4-dev
cargo install cargo-deb
```

`libnm-dev`, `libgtk-3-dev` and `libgtk-4-dev` are only needed for the editor libraries. The rest
is needed to build the workspace.

All commands below run from `nym-vpn-core/`.

## Build the plugin service

```sh
cargo build -p nym-nm-vpn-plugin
```

Run the unit tests with:

```sh
cargo test -p nym-nm-vpn-plugin
```

## Build the editor libraries

```sh
make -C crates/nym-nm-vpn-plugin/editor all test
```

`all` builds the three libraries into `crates/nym-nm-vpn-plugin/editor/build/`. `test` loads them
the way libnm and the GTK editors do and checks the `.name` file. The GTK tests print
`editor skipped: no display` when no display is available.

`make -C crates/nym-nm-vpn-plugin/editor clean` removes the build directory.

## Build the Debian package

Both parts have to be built first: `cargo deb` only collects the files. Packages
are always built from the release profile, since `cargo deb` does not support the debug one.

```sh
cargo build --release -p nym-nm-vpn-plugin
make -C crates/nym-nm-vpn-plugin/editor all test
cargo deb --no-build -p nym-nm-vpn-plugin
```

The package lands in `target/debian/nym-vpn-networkmanager_<version>_<arch>.deb`. If the editor
libraries are missing, `cargo deb` fails with a missing asset error. If they are stale, the
package silently contains the old ones, so rerun `make` after changing anything in `editor/`.

The package depends on `nym-vpnd`, which is built and packaged the same way with
`cargo deb -p nym-vpnd`.

## Install and try it

```sh
sudo apt-get install --reinstall ./target/debian/nym-vpn-networkmanager_*.deb
sudo systemctl restart NetworkManager
```

Restarting NetworkManager makes it reload the plugin libraries and the `.name` file. The package
installs:

| Path | What it is |
| --- | --- |
| `/usr/libexec/nm-nymvpn-service` | Plugin service |
| `/usr/libexec/nm-nymvpn-auth-dialog` | Auth dialog that reports no secrets, so GNOME does not stall asking for them |
| `/usr/lib/NetworkManager/VPN/nm-nymvpn-service.name` | Registers the VPN type with NetworkManager |
| `/usr/lib/NetworkManager/*.so` | Editor libraries |
| `/usr/share/dbus-1/system.d/nm-nymvpn-service.conf` | D-Bus policy for the service |
| `/usr/lib/NetworkManager/system-connections/NymVPN.nmconnection` | The NymVPN profile |

Check that NetworkManager sees the profile and bring it up:

```sh
nmcli connection show NymVPN
nmcli connection up NymVPN
```

NetworkManager starts the service when the VPN is activated, and the service exits after 20 seconds
when idle. To pick up a new build of the service without restarting NetworkManager, bring the
connection down and stop the old process:

```sh
sudo nmcli connection down NymVPN
sudo pkill -f /usr/libexec/nm-nymvpn-service
```

## Logs

The service logs to stderr, which NetworkManager forwards to its journal:

```sh
journalctl -u NetworkManager -f
```

The log level follows `RUST_LOG` and otherwise defaults to `info`. To run the service by hand with
debug logs, stop NetworkManager's copy first (it owns the same bus name):

```sh
sudo /usr/libexec/nm-nymvpn-service --verbose
```
