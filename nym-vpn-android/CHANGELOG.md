# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Locale for high/low copy on Details screen (https://github.com/nymtech/nym-vpn-client/pull/4274)

### Changed
- Replace mnemonic and access code with passphrase (https://github.com/nymtech/nym-vpn-client/pull/4274)
- Replace gateway with server (https://github.com/nymtech/nym-vpn-client/pull/4274)

## [2.6.0] - 2025.12.18

### Added
- Add Exit and Entry points to notification (https://github.com/nymtech/nym-vpn-client/pull/4133)
- Let user view Server details from home screen (https://github.com/nymtech/nym-vpn-client/pull/4163)
- Add Custom DNS settings (https://github.com/nymtech/nym-vpn-client/pull/4189)
- Add restartTunnel with proper VpnService lifecycle handling (https://github.com/nymtech/nym-vpn-client/pull/4237)

### Changed
- Reorder for Censorship screen sections (https://github.com/nymtech/nym-vpn-client/pull/4131)
- More informative logs with steps for tunnel configuration (https://github.com/nymtech/nym-vpn-client/pull/4162)
- Refactor service CompletableDeferred with StateFlow to fix VPN restart race (https://github.com/nymtech/nym-vpn-client/pull/4237)

### Fixed
- Add restricted apps for tunnel configurations (https://github.com/nymtech/nym-vpn-client/pull/4162)
- Fix for connection timer random reset (https://github.com/nymtech/nym-vpn-client/pull/4174)
- Fir for QUIC reconnection logic (https://github.com/nymtech/nym-vpn-client/pull/4237)

## [2.5.0] - 2025-12-03

### Added
- Node details added in quick settings notification (https://github.com/nymtech/nym-vpn-client/pull/4029)
- AmneziaWG section added to censorship screen (https://github.com/nymtech/nym-vpn-client/pull/4033)
- Revoke implementation added for VPN service (https://github.com/nymtech/nym-vpn-client/pull/4041)
- Account screen added (https://github.com/nymtech/nym-vpn-client/pull/4060)
- Android: Bi-mode Split Tunneling (https://github.com/nymtech/nym-vpn-client/pull/4049
- "Help with translation" link added to Languages screen (https://github.com/nymtech/nym-vpn-client/pull/4086)
- Allow user to logout when tunnel is Up (https://github.com/nymtech/nym-vpn-client/pull/4103)

### Changed
- UI changes and updates for Settings screen (https://github.com/nymtech/nym-vpn-client/pull/4060)
- Fixes for Privacy & Censorship screens (https://github.com/nymtech/nym-vpn-client/pull/4060)
- UI update for Navigation bar style (https://github.com/nymtech/nym-vpn-client/pull/4060)
- UI updates for Languages screen, Censorship screen, Create account screen (https://github.com/nymtech/nym-vpn-client/pull/4086)

### Fixed
- Connection error notification cleared after successfully connecting (https://github.com/nymtech/nym-vpn-client/pull/4029)
- Fix connection drops after language change (https://github.com/nymtech/nym-vpn-client/pull/4034)
- Fix QUIC autostart issue (https://github.com/nymtech/nym-vpn-client/pull/4040)
- Fix for Sentry toggle (https://github.com/nymtech/nym-vpn-client/pull/4060)

## [2.4.0] - 2025-11-19

### Added
- Crowdin Translations (https://github.com/nymtech/nym-vpn-client/pull/2777)

### Changed
- IAP enabled

### Fixed
- Split by US state and allow to connect to US state (https://github.com/nymtech/nym-vpn-client/pull/3895)
- Remove VerticalDivider for gateway trailing content (https://github.com/nymtech/nym-vpn-client/pull/3912)
- Fixes for servers display (https://github.com/nymtech/nym-vpn-client/pull/3911)

## [2.3.0] - 2025-11-06

### Added

- Add QUIC status and server description (https://github.com/nymtech/nym-vpn-client/pull/3696)
- Add streaming icon on home and Exit + update tooltip (https://github.com/nymtech/nym-vpn-client/pull/3774)
- Added In App purchases (https://github.com/nymtech/nym-vpn-client/pull/3754)
- Passphrase screen added (https://github.com/nymtech/nym-vpn-client/pull/3754)
- Account creation flow added (https://github.com/nymtech/nym-vpn-client/pull/3754)
- Android QUIC feature support (https://github.com/nymtech/nym-vpn-client/pull/3798)
- Modal dialogs for Logs screen (https://github.com/nymtech/nym-vpn-client/pull/3800)

### Changed

- Android: On Entry/Exit screens, display city instead of server key ID (https://github.com/nymtech/nym-vpn-client/pull/3733)
- In "Gateway details", use relative time in last update. (https://github.com/nymtech/nym-vpn-client/pull/3748)

### Fixed

- Display server up-time. (https://github.com/nymtech/nym-vpn-client/pull/3749)
- UI updates and changes (https://github.com/nymtech/nym-vpn-client/pull/3754)

## [2.1.0] - 2025-09-27

### Added

- Themed icons support (https://github.com/nymtech/nym-vpn-client/pull/3429)
- Random option added for EntryPoint (https://github.com/nymtech/nym-vpn-client/pull/3448)
- Server details screen (https://github.com/nymtech/nym-vpn-client/pull/3472)
- UI for Domain fronting (https://github.com/nymtech/nym-vpn-client/pull/3490)
- UI for QUIC (https://github.com/nymtech/nym-vpn-client/pull/3490)
- Feature flag support (https://github.com/nymtech/nym-vpn-client/pull/3490)

### Changed

- Detailed info status for Connecting state (https://github.com/nymtech/nym-vpn-client/pull/3448)
- Server name now displayed below Country on Main Screen (https://github.com/nymtech/nym-vpn-client/pull/3465)
- App metadata changed for Play Store (https://github.com/nymtech/nym-vpn-client/pull/3537)

### Fixed

- UI state updates after logout (https://github.com/nymtech/nym-vpn-client/pull/3448)
- UI fixes for (https://github.com/nymtech/nym-vpn-client/pull/3490)

## [2.0.0] - 2025-09-10

### Added

- Disengage the kill switch screens and functionality (https://github.com/nymtech/nym-vpn-client/pull/3360)

### Changed

- Updated error messages (https://github.com/nymtech/nym-vpn-client/pull/3354)
- Updated connection data (https://github.com/nymtech/nym-vpn-client/pull/3354)

### Fixed

- Fix the backstack for Welcome screen (https://github.com/nymtech/nym-vpn-client/pull/3366)

## [1.9.1] - 2025-08-29

### Fixed

- Changes and fixes for F-Droid

## [1.9.0] - 2025-08-27

### Added

- Download button for Logs Screen (https://github.com/nymtech/nym-vpn-client/pull/3197)
- Privacy & Data Screen (https://github.com/nymtech/nym-vpn-client/pull/3237)
- Network Monitoring flow (https://github.com/nymtech/nym-vpn-client/pull/3237)
- Welcome screen (https://github.com/nymtech/nym-vpn-client/pull/3256)

### Changed

- Updated UI for Modals (https://github.com/nymtech/nym-vpn-client/pull/3237)
- Changes and updates for SnackBar (https://github.com/nymtech/nym-vpn-client/pull/3237)
- Updates for Data Store (https://github.com/nymtech/nym-vpn-client/pull/3256)

### Fixed

- Fix for hardware crashes for specific devices (https://github.com/nymtech/nym-vpn-client/pull/3282)
- Fix for log reader crash (specific devices) (https://github.com/nymtech/nym-vpn-client/pull/3282)
- Fixes for Main Screen modals (https://github.com/nymtech/nym-vpn-client/pull/3325)

## [1.8.0] - 2025-07-31

### Fixed

- Clear backstack after login (https://github.com/nymtech/nym-vpn-client/pull/3102)
- Minor UI and routing fixes

## [1.7.0] - 2025-07-18

### Added

- Battery optimization prompt to guide users in improving background performance (https://github.com/nymtech/nym-vpn-client/pull/3024)
- Separate tabs for logs to enhance readability and organization (https://github.com/nymtech/nym-vpn-client/pull/3060)

### Fixed

- Crashes and ANRs reported by the Google Play Store (https://github.com/nymtech/nym-vpn-client/pull/3016)

### Changed

- Improved connection stability and reliability (https://github.com/nymtech/nym-vpn-client/pull/3017)
