# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- When navigating from the _Server details_ screen back to the node list,
  restore previous expanded nodes and scroll to the last focused node
- [Windows] Improve Windows installer when vpnd service fails
  to install, uninstall or start

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
