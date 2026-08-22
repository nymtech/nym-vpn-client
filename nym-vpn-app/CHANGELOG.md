# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2026.12.1] - 2026-08-22

### Added

- Allow marking specific server, country or region as favorite
- Add Safest server option for entry and exit
- Add support for Recents - recently connected servers

### Fixed

- Handle `NeedsDeviceLocation` error state

## [2026.11.0] - 2026-07-10

### Added

- Mixnet tuning settings
- Node families support
- Add new "Geo exclusion" settings to allow bypass the VPN tunnel when accessing from certain geographic regions. Currently only China is supported
- Add account refresh button to account settings page
- Debug logging settings toggle

### CHANGED

- Switch to daily allowance usage from monthly

## [2026.10.0] - 2026-06-09

### Added

- Allow adding custom apps (user selected) to split tunneling for Windows & Linux

## [1.30.0] - 2026-05-29

### Fixed

- Correctly handle Kosovo based nodes

### Added

- New UI 🚀
- Domain fronting (stealth api) toggle
- (linux) Prompt user for app restart after update

## [1.29.4] - 2026-05-07

## [1.29.3] - 2026-05-04

## [1.29.2] - 2026-05-01

## [1.29.0] - 2026-04-29

## [1.28.0] - 2026-04-14

### Added

- Split tunneling for Linux

## [1.28.0] - 2026-04-07

### Added

- Lewes protocol settings toggle
- Split tunneling for Windows

## [1.27.0] - 2026-03-31

### Added

- Autologin new user to the app upon account creation on web

## [1.26.0] - 2026-03-17

### Changed

- RPC client connection retries are triggered by UI

### Added

- Autologin user when navigating to web

## [1.25.0] - 2026-03-02

### Added

- Ad blocker
- Dynamic tray icon and menu reflecting current tunnel configuration
- See bandwidth usage and subscription expiry/renewal date
- Diagnostic settings

### Fixed

- Improve Accessibility for sliders
- Fix privy social linking check

## [1.24.0] - 2026-02-17

### Added

- Support for rtl languages
- Support account linking

### Changed

- Cleanup languages list

### Fixed

- Add missing translations

## [1.23.0] - 2026-02-02

### Added

- Privy social login
- Lewes Protocol switch in DEV settings

### Fixed

- UI styling

## [1.22.0] - 2026-01-16

## [1.22.0] - 2026-01-16

### Added

- Add new onboarding screens

## [1.21.0] - 2025-12-18

### Added

- Add custom DNS settings

### Changed

- Allow operating SOCKS5/RPC proxy independently from the main tunnel

## [1.20.3] - 2025-12-09

## [1.20.0] - 2025-12-02

### Added

- Allow seeing server details from home screen
- Auto focus node list search input
- Minimal obfuscation (AmneziaWG)
- Allow to connect dApps / wallets to the mixnet via SOCKS5 url or HTTP RPC url

### Changed

- Always show Anti-Censorship setting option
- Visual improvements

### Fixed

- Styling fixes and visual improvements

## [1.19.0] - 2025-11-19

### Added

- Add Greek, Bengali and Vietnamese translation languages
- Show "QUIC" label on the selected node only when truly connected
  over QUIC protocol
- Add a label to servers with a residential IP

### Changed

- Use custom icons for server score levels (good, normal, poor)
- Migrate daemon communication layer to RpcClient

## [1.18.0] - 2025-11-03

### Added

- Filter server list by QUIC protocol when QUIC mode is enabled in the settings,
  entry node only
- Add QUIC tags in server list and in home screen on the entry node input
- Add server description and QUIC support to server details screen
- Add US state as selectable location for entry and exit nodes
- Group US servers by states in server list
- Show server location like country, state and city in various places in the UI
- Sort server list by score when searching or filtering
- Search by city, region and server ID in server list
- Optimize server list filtering, searching and rendering performance

### Fixed

- Fix a race condition at app startup that could lead to
  selected nodes to be reset to default
- Fix incorrect JS kv store API types

## [1.17.1] - 2025-10-20

### Fixed

- Fix wrong binary version, bump to 1.17.1

## [1.17.0] - 2025-10-17

### Added

- When navigating from the _Server details_ screen back to the node list,
  restore previous expanded nodes and scroll to the last focused node
- [Windows] Improve NSIS installer when vpnd service fails
  to install, uninstall or start, show interactive dialog messages

### Fixed

- Fix initial scroll position when navigate to _Server details_ screen

## [1.16.0] - 2025-09-26

### Added

- Show the picked gateway name the tunnel is connected to, when
  connecting/connected and the selected node is a country
- Add QUIC mode and Domain-fronting (aka stealth API) settings options
  Note: disabled for now
- Add a new screen for gateway details

### Fixed

- [Linux] Fix deb package version for prerelease builds, and description

## [1.15.0] - 2025-09-10

### Added

- Add event progress messages while connecting, showing the steps daemon is
  performing

### Changed

- Max and min window size adjusted to support exotic screen resolutions
- Refactor UI early state initialization

### Fixed

- [Windows] Installer fails to uninstall daemon during upgrade
- Fix in-app update popup not showing after the Welcome screen
- Fix error window not opening on startup error

## [1.14.0] - 2025-08-26

### Added

- Add _Anonymous network stats_ toggle to the Welcome screen
- Add live update support of internal user account state from daemon (Account Controller)
- Add a UX flow to invite the user to select a plan (new subscription or renew)
- Show a message in UI when the user account has no active subscription
- Show a message in UI when the user account has exceeded its bandwidth usage
- Display the internal state of the user account in the settings
- [Windows] Add splash animation

### Changed

- Improve UI emphasis when connection to the daemon is down

### Fixed

- [Linux] Fix task-bar window icon
- [Linux] Fix window bar buttons (close, minimize, maximize) on Wayland

## [1.13.0] - 2025-07-30

### Added

- Add an option in the settings allowing to disable IPv6 support
- Add a new setting menu "Privacy and data" to control error monitoring
  and network statistics collection
- Remove libfuse2 dependency (EOL) on Linux AppImage

## [1.12.0] - 2025-07-18

### Added

- Add Sentry error monitoring at daemon level, controlled by
  the existing toggle in app settings (OFF by default)

### Changed

- Add additional system information in logs to help with debugging
- Update dependencies to latest versions

## [1.11.0] - 2025-06-18

### Added

- [Windows] Add auto-updater
