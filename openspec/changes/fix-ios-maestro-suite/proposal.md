## Why

The iOS Maestro UI test suite (`nym-vpn-apple/maestro`) is broken and stale. The mock build (`NymVPN-Mock`) fails to compile at package-graph resolution, and the 7 existing flows target a UI that no longer exists after the onboarding/home redesign. Android already received this treatment (rewritten flows, aligned mock backend); iOS never did, so its `ci-maestro-ios` job cannot pass. This change brings iOS to parity so it protects the redesigned UI the same way Android does.

## What Changes

- **Unblock the mock build.** Lower `nym-vpn-apple/Routes/Package.swift` from `swift-tools-version: 6.2` to `5.10`. The 6.2 declaration forces Swift 6.2 / Xcode 26 tooling, so `xcodebuild build-for-testing -scheme NymVPN-Mock` fails at *Resolve Package Graph* on any Swift 6.1 toolchain (local machines and the unpinned CI runner) before code compiles. `Routes` is a single trivial `Codable` enum with no 6.2 features; its ten sibling packages use 5.9/5.10; deployment target is iOS 16/17 with no iOS 26 SDK usage — the bump was gratuitous.
- **Verify and align the mock backend.** Once compilation is reachable, confirm `MockConnectionState` / `MockMode` (`Services/.../MockConnectionManager.swift`) and the `CredentialsManager` mock paths still compile against current interfaces; fix any drift, mirroring the Android mock-alignment work.
- **Rewrite the 7 existing flows for the redesigned UI.** Replace stale copy assertions (`"Welcome to NymVPN"`, `"Disconnected"`, `"Get started"`) and brittle selectors (`point: "95%,12%"`, `visible: ".*"`) with text selectors against the redesign's actual copy (`"Welcome to Nym VPN!"`, `"Connect"`/`"Connecting"`/`"Disconnect"`, `"Entry server"`/`"Exit server"`, `"Welcome!"`/`"Continue"`), mirroring Android's text-based style.
- **Restructure to mirror Android.** Introduce reusable `subflows/` (e.g. `open_app`, `connect`) and per-screen flow folders (`main_screen/`, `nodes/`, `settings_screen/`, `login/`).
- **Expand to Android parity.** Grow from 7 flat flows toward Android's ~17: node search, node info, settings navigation/components, theme switch, customize DNS, anti-censorship, split tunneling, passphrase, and login/create-account navigation — each backed by an existing iOS redesign screen.
- **(Optional) Harden CI.** Pin the Xcode version in `ci-maestro-ios.yml` instead of relying on the unpinned default `/Applications/Xcode.app`.

## Capabilities

### New Capabilities
- `ios-ui-testing`: End-to-end Maestro UI validation of the redesigned iOS app running against the mock backend — a buildable `NymVPN-Mock` target and a maintained flow suite at parity with Android's coverage.

### Modified Capabilities
<!-- None — no existing OpenSpec specs; this is the first capability in this repo. -->

## Impact

- **Build config:** `nym-vpn-apple/Routes/Package.swift` (tools version).
- **Rust core build (discovered during implementation):** the mock links `NymVPNLib` (the gitignored Rust FFI core, 51 importing Swift files), so the build cannot resolve until the core is generated. `nym-vpn-core/iOS.mk` gains a simulator xcframework slice (`aarch64-apple-ios-sim`) so the mock runs on the simulator; the device archive is unaffected (additive).
- **CI:** `.github/workflows/ci-maestro-ios.yml` rewritten to build the core on an `AppleSilicon` runner (rust iOS + iOS-sim targets, cargo-swift, protoc, go → `make -f iOS.mk`, stage `NymVPNLib`) before `xcodebuild`. Previously it never built the core, so the job had never been buildable.
- **Mock backend (only if drift found):** `.../ConnectionManager/MockConnectionManager.swift`, `.../CredentialsManager/CredentialsManager.swift` — verifiable only once the CI build compiles.
- **Test suite:** `nym-vpn-apple/maestro/` — rewritten + expanded to 17 flows (Android parity), new `subflows/` and per-screen folders; `config.yaml` unchanged (`appId: net.nymtech.vpn`, `MOCK_MODE` launch).
- **Verification:** the flow suite is authored against source-resolved copy; it is verified in CI, **not locally** (this Intel Mac cannot build the device-only iOS core).
- **No app runtime/product code changes** beyond the one-line package tools version, the additive `iOS.mk` sim slice, and any mock-alignment fixes. No changes to Android, macOS product behavior, or the real daemon path.
