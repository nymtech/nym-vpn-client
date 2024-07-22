# nym-vpn-apple

👋 Hello 👋

Welcome to the home of NymVPN iOS + iPadOS.

Building instructions: 
```
1. Install required dependancies.
```

Dependancy requirements:
```
brew, go, swiftlint
```

Notices:
- If mac is connected to VPN, Xcode and device connection is not working.


macOS:

Sparkle auto updater instructions

CI job artefacts:
- `NymVPN.dmg` contains app in the `.dmg`. App and `.dmg` are signed with Developer ID cert, Notarized by Apple, signed with Sparkle EdDSA signature for auto-updates. The app and `.dmg` need to be named same - https://sparkle-project.org/documentation/publishing/#publishing-an-update
- `appcast.xml` Sparkle auto updater file, required for the auto updates to work. Replace URL with GH release url. Append content to existing `appcast.xml`.
