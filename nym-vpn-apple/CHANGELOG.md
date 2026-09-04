# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Removed

- Widget support for both iOS & macOS

## [2026.12.0] - TBD


### Fixed

- Fix switching between modes to always land on the last selected mode (https://github.com/nymtech/nym-vpn-client/pull/6023)
- Show full version in settings (https://github.com/nymtech/nym-vpn-client/pull/6032)
- Sync daemon settings on rpc connect. Fixes issues with stale configuration being displayed in UI (https://github.com/nymtech/nym-vpn-client/pull/6067)


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
