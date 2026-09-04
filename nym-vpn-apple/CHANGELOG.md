# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Removed

- Widget support for both iOS & macOS (https://github.com/nymtech/nym-vpn-client/pull/6255)

## [2026.12.3] - 2026-08-27

### Added

- macOS: launch app after pkg installation completes (https://github.com/nymtech/nym-vpn-client/pull/6205)

### Fixed

- Login processing stuck when setup carousel is interrupted (https://github.com/nymtech/nym-vpn-client/pull/6203)

## [2026.12.2] - 2026-08-25

### Fixed

- Show error when connection attempts are exceeded (https://github.com/nymtech/nym-vpn-client/pull/6178)

## [2026.12.1] - 2026-08-21

### Added

- Server row in Geo Exclusion settings, simplified setup instructions (https://github.com/nymtech/nym-vpn-client/pull/6148)

### Fixed

- Login processing bars stay in sequence with setup copy (https://github.com/nymtech/nym-vpn-client/pull/6152)
- Pre-auth drawer stranding on the Home screen (https://github.com/nymtech/nym-vpn-client/pull/6146)
- iOS: duplicate in-session account prepare after login; macOS: close control on critical snackbars (https://github.com/nymtech/nym-vpn-client/pull/6142)

## [2026.12.0] - 2026-08-18

### Added

- Recently used gateways section in server selection (https://github.com/nymtech/nym-vpn-client/pull/5911)
- Favorites for gateways, countries and regions (https://github.com/nymtech/nym-vpn-client/pull/5918)
- Auto and Safest options for entry and exit, with Random and Safest icons (https://github.com/nymtech/nym-vpn-client/pull/5962, https://github.com/nymtech/nym-vpn-client/pull/6009)
- New onboarding flow (https://github.com/nymtech/nym-vpn-client/pull/5964)

### Changed

- Default mixnet tunnel configuration name is now "NymVPN" (https://github.com/nymtech/nym-vpn-client/pull/5875)
- Connect circle color for mixnet mode (https://github.com/nymtech/nym-vpn-client/pull/6009)

## [2026.12.0] - TBD


### Fixed

- Fix switching between modes to always land on the last selected mode (https://github.com/nymtech/nym-vpn-client/pull/6023)
- Show full version in settings (https://github.com/nymtech/nym-vpn-client/pull/6032)
- Sync daemon settings on rpc connect. Fixes issues with stale configuration being displayed in UI (https://github.com/nymtech/nym-vpn-client/pull/6067)
- Handle `NeedsDeviceLocation` error and explain Safest routing errors (https://github.com/nymtech/nym-vpn-client/pull/6106)
- macOS: crash when logged out or during onboarding (https://github.com/nymtech/nym-vpn-client/pull/6082)
- Geo Exclusion port display format (https://github.com/nymtech/nym-vpn-client/pull/6070)
- macOS: favorites not persisted (https://github.com/nymtech/nym-vpn-client/pull/5929)


## [2.11.0]

### Added

- US region selection

### Fixed

- Country loosing selection state 

## [2.10.0] - 2025-10-02

### Added

- Connecting states
- Liquid glass icon
- Display server moniker below country in main view
- Improved server details view

### Fixed

- UI glitches in gateway/country list view
- macOS: daemon not able to update after vpnd service migration

## [2.9.0] - 2025-09-03

### Fixed

- macOS: window activation
- iOS: setting item subtitles partially visible on small screens

## [2.8.0] - 2025-07-31

### Added

- macOS: IPv6 toggle in settings

### Fixed

- Connection time jumping

## [2.7.0] - 2025-07-28

### Added

- In App Purchases functionality

### Fixed

- iOS: reconnect after changing mode or gateway/country
