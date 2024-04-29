```
${BUILD_INFO}
```

## Notes

Release build of the core binaries for the nym vpn client

The core binaries consist of

- `nym-vpn-cli`: Basic commandline client for running the vpn. This runs in the foreground.
- `nym-vpnd`: Daemon implementation of the vpn client that can run in the background and interacted with using `nym-vpnc`.
- `nym-vpnc`: The commandline client used to interact with `nym-vpnd`.

### Running

If you are running Debian/Ubuntu/PopOS or any other distributio supporting debian packages and systemd, see the relevant section below

#### Daemon

Start the daemon with

```sh
sudo -E ./nym-vpnd
```

Then

```sh
./nym-vpnc status
./nym-vpnc connect
./nym-vpnc disconnect
```

#### CLI

An alternative to the daemon is to run the `nym-vpn-cli` commandline client that runs in the foreground.
```sh
./nym-vpn-cli run
```

### Debian package for Debian/Ubuntu/PopOS

For linux platforms using deb packages and systemd, there is also debian packages. 

```sh
sudo apt install ./nym-vpnd_0.1.0-1_amd64.deb ./nym-vpnc_0.1.0-1_amd64.deb (substitute the correct versions)
```

Installing the `nym-vpnd` deb package starts a `nym-vpnd.service`. Check that the daemon is running with
```sh
systemctl status nym-vpnd.service
```
and check its logs with
```sh
sudo journalctl -u nym-vpnd.service -f
```
To stop the background service
```sh
systemctl stop nym-vpnd.service
```
It will start again on startup, so disable with
```sh
systemctl disable nym-vpnd.service
```

Interact with it with `nym-vpnc`
```sh
nym-vpnc status
nym-vpnc connect
nym-vpnc disconnect
```

## SHA256 Checksums

```
${SHA256_CHECKSUMS}
```
