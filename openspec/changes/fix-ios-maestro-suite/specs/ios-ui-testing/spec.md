## ADDED Requirements

### Requirement: Mock build compiles on the supported toolchain

The `NymVPN-Mock` scheme SHALL build for the iOS Simulator using the toolchain the CI workflow provides, without requiring a Swift toolchain newer than the project's other packages demand. No local Swift package MUST declare a `swift-tools-version` that exceeds what the app's compilation actually requires.

#### Scenario: Package graph resolves on Swift 6.1

- **WHEN** `xcodebuild build-for-testing -workspace NymVPN.xcworkspace -scheme NymVPN-Mock -destination 'platform=iOS Simulator,name=iPhone 16,OS=latest'` runs on a Swift 6.1 toolchain
- **THEN** the package graph resolves without a "package 'routes' is using Swift tools version 6.2.0 but the installed version is 6.1.0" error

#### Scenario: Mock app builds to an installable bundle

- **WHEN** the `NymVPN-Mock` scheme is built for testing with `SWIFT_ACTIVE_COMPILATION_CONDITIONS='MOCK_MODE'`
- **THEN** compilation succeeds and produces a `NymVPN.app` bundle installable on the simulator

### Requirement: Mock backend simulates connect/disconnect without the daemon

When launched in mock mode, the app SHALL drive UI state transitions from the mock backend so flows can exercise connect and disconnect without the real `nym-vpnd` daemon or network. The mock SHALL compile against the current `ConnectionManager` and `CredentialsManager` interfaces.

#### Scenario: Connect transitions through mock states

- **WHEN** the app is launched with `MOCK_MODE` and the user triggers a connect
- **THEN** the tunnel status transitions disconnected → connecting → connected using the mock, with no daemon dependency

#### Scenario: Credentials treated as present in mock mode

- **WHEN** the app runs in mock mode
- **THEN** credential-gated screens behave as if an account is stored, so flows reach the home screen without real login

### Requirement: Flow suite validates the redesigned UI

Every Maestro flow under `nym-vpn-apple/maestro/flows` SHALL assert against the redesigned UI's actual copy and MUST NOT rely on obsolete copy or on brittle absolute-coordinate taps (`point: "x%,y%"`) or wildcard visibility (`visible: ".*"`) where a stable text selector exists.

#### Scenario: Home screen renders in disconnected state

- **WHEN** the app launches in mock mode and onboarding is dismissed
- **THEN** the flow asserts the redesigned home controls are visible (e.g. `"Connect"`, `"Entry server"`, `"Exit server"`)

#### Scenario: No flow references removed copy

- **WHEN** the flow suite is reviewed
- **THEN** no flow asserts on removed strings such as `"Welcome to NymVPN"` or the old `"Disconnected"` landing copy, and no flow uses `point:` taps or `visible: ".*"` in place of an available text selector

### Requirement: Reusable flow structure mirrors Android

The iOS suite SHALL factor shared setup into `subflows/` (at minimum app launch and connect) and organize flows into per-screen folders, mirroring the Android suite's structure so the two platforms stay comparable.

#### Scenario: Shared setup is a subflow

- **WHEN** multiple flows need to launch the app and dismiss onboarding
- **THEN** they invoke a shared `subflows/` file rather than duplicating the launch steps

#### Scenario: Flows grouped by screen

- **WHEN** the suite is laid out
- **THEN** flows live under per-screen folders (e.g. `main_screen/`, `nodes/`, `settings_screen/`, `login/`) analogous to Android

### Requirement: Coverage reaches Android parity

The iOS flow suite SHALL cover the same user journeys Android covers, for every journey whose screen exists in the iOS redesign: app launch, connect/disconnect, mode selection, node selection, node search, node info, login/create-account navigation, settings navigation, theme switch, customize DNS, anti-censorship, and split tunneling.

#### Scenario: Parity journeys are present

- **WHEN** the iOS suite is compared against the Android suite
- **THEN** each Android-covered journey with an equivalent iOS screen has a corresponding iOS flow

#### Scenario: Suite runs green in CI

- **WHEN** `ci-maestro-ios` runs the flow suite against a booted simulator with the mock app installed
- **THEN** all flows pass and a JUnit report is produced
