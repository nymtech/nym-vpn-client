# nym-vpn-apple

👋 Hello 👋

Welcome to the home of NymVPN iOS + iPadOS + macOS.

Building instructions: 
```
1. Install required dependancies.
```

Dependancy requirements:
```
brew, go, swiftlint, swift-protobuf, grpc-swift
```

Notices:
- If mac is connected to VPN, Xcode and device connection is not working.

iOS:

Mixnet lib update:

1. `nym-vpn-client/nym-vpn-apple/MixnetLibrary/Package.swift` update the binary target url and checksum.
2. Download the source from releases. 
3. Update `nym-vpn-client/nym-vpn-apple/MixnetLibrary/Sources/MixnetLibrary/nym_vpn_lib.swift` with `nym-vpn-client-nym-vpn-core-vx.x.x/nym-vpn-core/nym-vpn-lib/uniffi/nym_vpn_lib.swift`  

macOS:

GRPC client update:
1. Download `nym-vpn-core-vx.x.x_macos_universal.tar.gz`
2. Rename `nym-vpnd` to `net.nymtech.vpn.helper`
3. Update `nym-vpn-client/nym-vpn-apple/Daemon/net.nymtech.vpn.helper`
4. Optionally: `brew install swift-protobuf` && `brew install grpc-swift`
5. cd `nym-vpn-client-nym-vpn-core-vx.x.x/proto/nym`
6. `protoc --swift_out=. vpn.proto` && `protoc --grpc-swift_out=. vpn.proto`
7. Update files from `nym-vpn-client-nym-vpn-core-vx.x.x/proto/nym` to `nym-vpn-client/nym-vpn-apple/ServicesMacOS/Sources/GRPCManager/proto/nym`
8. Disable swiftlint `// swiftlint:disable all` for `.proto`, `.grpc` , `.pb` files.

Sparkle auto updater instructions

CI job artefacts:
- `NymVPN.dmg` contains app in the `.dmg`. App and `.dmg` are signed with Developer ID cert, Notarized by Apple, signed with Sparkle EdDSA signature for auto-updates. The app and `.dmg` need to be named same - https://sparkle-project.org/documentation/publishing/#publishing-an-update
- `appcast.xml` Sparkle auto updater file, required for the auto updates to work. Replace URL with GH release url. Append content to existing `appcast.xml`.
