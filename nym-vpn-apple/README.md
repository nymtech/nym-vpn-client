# nym-vpn-apple

👋 Hello 👋

Welcome to the home of [NymVPN](https://nymvpn.com/en) iOS + iPadOS + macOS. The Android client application for [NymVPN](https://nymvpn.com/en). For more information about NymVPN, its features, latest announcements, Help Center, or to download the latest stable release, visit [nymvpn.com](https://nymvpn.com/en).


## Building instructions: 
```
1. Install required dependancies.
```
### Dependancy requirements:
```
brew, go, swiftlint, swift-protobuf, grpc-swift
```
## Notices:

- If mac is connected to VPN, Xcode and device connection is not working.


## Env vars:
### Update the env vars from https://github.com/nymtech/nym/tree/develop/envs

```
sh Scripts/UpdateEnvVars.sh
```
## Lib/daemon update
### Automatic core update
To update core daemon/lib automatically, run script from `Scripts` dir.
```
sh UpdateCore.sh 0.0.1
```
Updates
- MixnetLibrary Package url + checksum + uniffy swift file
- daemon, generates proto files

### iOS Mixnet lib manual update:
1. `nym-vpn-client/nym-vpn-apple/MixnetLibrary/Package.swift` update the binary target url and checksum.
2. Download the source from releases.
3. Update `nym-vpn-client/nym-vpn-apple/MixnetLibrary/Sources/MixnetLibrary/nym_vpn_lib.swift` with `nym-vpn-client-nym-vpn-core-vx.x.x/nym-vpn-core/nym-vpn-lib/uniffi/nym_vpn_lib.swift`

### macOS GRPC client manual update:
1. Download `nym-vpn-core-vx.x.x_macos_universal.tar.gz`
2. Rename `nym-vpnd` to `net.nymtech.vpn.helper`
3. Update `nym-vpn-client/nym-vpn-apple/Daemon/net.nymtech.vpn.helper`
4. Optionally: `brew install swift-protobuf` && `brew install grpc-swift`
5. cd `nym-vpn-client-nym-vpn-core-vx.x.x/proto/nym`
6. `protoc --swift_out=. vpn.proto` && `protoc --grpc-swift_out=. vpn.proto`
7. Update files from `nym-vpn-client-nym-vpn-core-vx.x.x/proto/nym` to `nym-vpn-client/nym-vpn-apple/ServicesMacOS/Sources/GRPCManager/proto/nym`
8. Disable swiftlint `// swiftlint:disable all` for `.proto`, `.grpc` , `.pb` files.

## Debugging daemon with extra logs:
Remove installed daemon from `/Library/LaunchDaemons` and `/Library/PrivilegedHelperTools`. Kill the process in `Activity monitor`. Comment out `isHelperInstalled()` call.

Run daemon with logs:
`sudo RUST_LOG=debug ./net.nymtech.vpn.helper`


## Sparkle auto updater instructions
### CI job artefacts:
- `NymVPN.dmg` contains app in the `.dmg`. App and `.dmg` are signed with Developer ID cert, Notarized by Apple, signed with Sparkle EdDSA signature for auto-updates. The app and `.dmg` need to be named same - https://sparkle-project.org/documentation/publishing/#publishing-an-update

- `appcast.xml` Sparkle auto updater file, required for the auto updates to work. Replace URL with GH release url. Append content to existing `appcast.xml`.
