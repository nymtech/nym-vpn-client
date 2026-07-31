## 1. Unblock the mock build

- [x] 1.1 Change `nym-vpn-apple/Routes/Package.swift` line 1 from `swift-tools-version: 6.2` to `5.10` — done; the `Routes 6.2` resolution error is eliminated
- [ ] 1.2 Run `xcodebuild build-for-testing -workspace NymVPN.xcworkspace -scheme NymVPN-Mock -destination 'platform=iOS Simulator,name=iPhone 16,OS=latest' -derivedDataPath build CODE_SIGNING_ALLOWED=NO SWIFT_ACTIVE_COMPILATION_CONDITIONS='MOCK_MODE'` and confirm package resolution succeeds
- [ ] 1.3 Confirm `Routes` source (`HomeLink.swift`) still compiles under 5.10

## 2. Verify and align the mock backend

- [ ] 2.1 Let the build proceed past resolution and capture any compiler errors
- [ ] 2.2 If `MockConnectionState` / `MockMode` (`Services/.../MockConnectionManager.swift`) fail against current `ConnectionManager`, align them (mirror Android mock-alignment)
- [ ] 2.3 If the `CredentialsManager` mock paths fail, align them so credential-gated screens behave as account-present in mock mode
- [ ] 2.4 Confirm the build produces an installable `NymVPN.app` (Debug-iphonesimulator) with `MOCK_MODE`

## 3. Establish shared flow structure

- [x] 3.1 Create `nym-vpn-apple/maestro/subflows/open_app.yaml` (launch with `MOCK_MODE`, dismiss onboarding via `"Get started"` / `"Close"`, wait for home)
- [x] 3.2 Create `nym-vpn-apple/maestro/subflows/connect.yaml` (tap `"Connect"`, wait through `"Connecting"` to connected state)
- [x] 3.3 Add any additional shared subflow needed for reset/logout, matching Android's `logout_to_welcome`
- [x] 3.4 Create per-screen folders: `flows/main_screen/`, `flows/nodes/`, `flows/settings_screen/`, `flows/login/`

## 4. Rewrite the 7 existing flows for the redesigned UI

- [x] 4.1 `app_launch` — assert redesigned home controls (`"Connect"`, `"Entry server"`, `"Exit server"`), remove `point:` taps and `visible: ".*"`
- [x] 4.2 `connect_disconnect` — full connect→disconnect round trip via subflows and `"Connect"`/`"Connecting"`/`"Disconnect"`
- [x] 4.3 `connect_modes` — mode selection against redesigned copy
- [x] 4.4 `login` — welcome/auth entry using `"Welcome!"` / `"Continue"` (drop `"Log in"` / `"Create account"` old copy)
- [x] 4.5 `login_mnemonic` — passphrase sign-in path against `PassphraseSignIn` screen
- [x] 4.6 `node_selection` — entry/exit server selection against `GatewaysView`
- [x] 4.7 `settings_navigation` — settings entry/navigation against `SettingsView`
- [ ] 4.8 Verify each rewritten flow on the simulator — DEFERRED: not runnable on this Intel Mac (needs Rust core + Apple Silicon); verifies in CI

## 5. Expand to Android parity

- [x] 5.1 `nodes/node_search` — search a gateway via `SearchView` / `GatewaysView`
- [x] 5.2 `nodes/node_not_found_search` — empty/no-result search state
- [x] 5.3 `nodes/node_info` — authored as gateway-list validation; ServerDetails step left as a CI-finalization TODO (info-button selector unresolvable from source)
- [x] 5.4 `login/create_account_navigation` — create-account navigation path
- [x] 5.5 `settings_screen/settings_components` — settings rows/components render
- [x] 5.6 `settings_screen/theme_switch` — appearance/app-mode via `AppearanceView` / `AppModeView`
- [x] 5.7 `settings_screen/customize_dns` — DNS settings via `DnsView`
- [x] 5.8 `settings_screen/anti_censorship` — censorship settings via `CensorshipView`
- [x] 5.9 `settings_screen/split_tunnelling` — split tunnel via `SplitTunnelView`
- [x] 5.10 `settings_screen/passphrase` — passphrase screen via `PassphraseView`
- [x] 5.11 Log any Android journey that has no iOS redesign equivalent and is intentionally omitted
- [ ] 5.12 Verify each parity flow on the simulator — DEFERRED to CI (see 4.8)

## 6. Rust core prerequisite (discovered during apply)

The mock app links `NymVPNLib` (the Rust FFI core), which is gitignored and absent, so resolution fails after the Routes fix. Even the mock must build the core. This machine (Intel, device-only `iOS.mk`, no iOS rust targets) cannot build it, so verification moves to CI on Apple Silicon.

- [x] 6.1 `nym-vpn-core/iOS.mk` — build the simulator slice too (`--target aarch64-apple-ios-sim`) so `NymVPNLibUniffi.xcframework` runs on the simulator; device archive unaffected (additive)
- [ ] 6.2 Confirm the device release build (`build-nym-vpn-apple.yml`) still succeeds with the added sim slice

## 7. CI and final validation

- [x] 7.1 Rewrite `.github/workflows/ci-maestro-ios.yml`: run on `AppleSilicon`, load `.env` versions, install rust (`aarch64-apple-ios` + `-sim`), cargo-swift, protoc, go; build `iOS.mk` and stage `NymVPNLib` before `xcodebuild`
- [ ] 7.2 Confirm the rewritten `ci-maestro-ios` builds the core, builds+installs the mock, runs the suite, and uploads the JUnit report (first real end-to-end run)
- [ ] 7.3 In that CI run, resolve the `node_info` info-button selector and complete flow 5.3; confirm mock-backend compiles (Group 2) or fix drift surfaced by the compiler
- [ ] 7.4 (Optional) Pin the Xcode version on the self-hosted runner if its default drifts
