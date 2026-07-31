# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2026.12.0] - TBD

### Added
- Recents support (https://github.com/nymtech/nym-vpn-client/pull/5922)
- Favorites support (https://github.com/nymtech/nym-vpn-client/pull/5922)
- Onboarding screen (https://github.com/nymtech/nym-vpn-client/pull/5968)

### Changed
- Changed UI for Server Details screen (https://github.com/nymtech/nym-vpn-client/pull/5922)
- Changed UI for Server List screen (https://github.com/nymtech/nym-vpn-client/pull/5922)
- Changed Mixnet tuning appearance in Settings (https://github.com/nymtech/nym-vpn-client/pull/5958)
- Update UI and sections for Mixnet Tuning screen (https://github.com/nymtech/nym-vpn-client/pull/5958)

### Fixed
- Fix auto-connect getting stuck after boot (https://github.com/nymtech/nym-vpn-client/pull/5924)
- Fix Search across favorites (https://github.com/nymtech/nym-vpn-client/pull/5958)
- Fix for Geo Exclusion UI (https://github.com/nymtech/nym-vpn-client/pull/5958)

## [v2026.11.3]

### Added
- Add 32-bit arch support (https://github.com/nymtech/nym-vpn-client/pull/5881)

### Fixed
- Fixes for old Android versions (https://github.com/nymtech/nym-vpn-client/pull/5884)

## [v2026.11.1]

### Added
- Add Icon Switcher to Appearance settings. With camouflage options (https://github.com/nymtech/nym-vpn-client/pull/5837)

### Changed
- Update Colors for light theme (https://github.com/nymtech/nym-vpn-client/pull/5843)

### Fixed
- Fix for the issue with bottom sheet dialogs overlapping (https://github.com/nymtech/nym-vpn-client/pull/5843)

## [v2026.11.0] - 10.07.2026

### Added
- Add allowed for Split Tunneling system apps list (https://github.com/nymtech/nym-vpn-client/pull/5563)
- Handle another VPN detection (https://github.com/nymtech/nym-vpn-client/pull/5570)
- Add Notifications screen (https://github.com/nymtech/nym-vpn-client/pull/5609)
- Add Node Families support (https://github.com/nymtech/nym-vpn-client/pull/5609)
- Add Geo Exclusion feature (https://github.com/nymtech/nym-vpn-client/pull/5626)
- Add UI for Setup Instructions screen (https://github.com/nymtech/nym-vpn-client/pull/5626)
- Add changes to the core sender for Geo Exclusion (https://github.com/nymtech/nym-vpn-client/pull/5626)

### Changed
- Enable Mixnet Tuning (https://github.com/nymtech/nym-vpn-client/pull/5563)
- Change AccountDetails.kt UI to match daily allowance (https://github.com/nymtech/nym-vpn-client/pull/5604)

## [v2026.10.0] - 09.06.2026

### Added
- Add local account creation flow for de-googled version (https://github.com/nymtech/nym-vpn-client/pull/5474)

### Changed
- Remove unused strings from strings.xml (https://github.com/nymtech/nym-vpn-client/pull/5480)
- Code clean up (https://github.com/nymtech/nym-vpn-client/pull/5480)
- Change colors for Split Tunneling screen (https://github.com/nymtech/nym-vpn-client/pull/5482)
- Change .apk naming format to match F-droid (https://github.com/nymtech/nym-vpn-client/pull/5482)

### Fixed
- Fix VPN revoke race and idle disconnect (https://github.com/nymtech/nym-vpn-client/pull/5483)

## [3.5.0] - 01.06.2026

### Added
- Add 1-click (https://github.com/nymtech/nym-vpn-client/pull/5236)
- Add auth view (https://github.com/nymtech/nym-vpn-client/pull/5236)
- Add code formatting (https://github.com/nymtech/nym-vpn-client/pull/5236)
- Enable domain fronting toggle (https://github.com/nymtech/nym-vpn-client/pull/5272)
- Add checks for Private DNS state and show dialog when it's not off when turning on the Ad-block toggle (https://github.com/nymtech/nym-vpn-client/pull/5308)
- Add pulsing effect for app logo (https://github.com/nymtech/nym-vpn-client/pull/5308)
- Add alert when subscription expired (https://github.com/nymtech/nym-vpn-client/pull/5416)

### Changed
- Update App theme, colors, typography (https://github.com/nymtech/nym-vpn-client/pull/5236)
- Clean up unused screens (https://github.com/nymtech/nym-vpn-client/pull/5236)
- Remove reconnect on Ad-block toggle (https://github.com/nymtech/nym-vpn-client/pull/5269)
- Remove PQ toggle (https://github.com/nymtech/nym-vpn-client/pull/5272)
- Allow back press on Passphrase screen when passphrase revealed (https://github.com/nymtech/nym-vpn-client/pull/5276)
- Update App icons (https://github.com/nymtech/nym-vpn-client/pull/5416)

### Fixed
- Fix spinner position for Auto login dialog (https://github.com/nymtech/nym-vpn-client/pull/5287)
- Update Connect panel to fix the positions of server info when connected (https://github.com/nymtech/nym-vpn-client/pull/5308)
- Fix jumping logo on Splash screen (https://github.com/nymtech/nym-vpn-client/pull/5308)
- Fix Connection Status view jumps when the connection time showed (https://github.com/nymtech/nym-vpn-client/pull/5308)
- Fix wrapping of app count label on Split tunneling screen (https://github.com/nymtech/nym-vpn-client/pull/5308)
- Don't allow user to connect when subscription expired (https://github.com/nymtech/nym-vpn-client/pull/5416)

## [3.4.0] - 8.05.2026

### Added
- Add Ad blocking (https://github.com/nymtech/nym-vpn-client/pull/5033)
- Add Pending subscription status (https://github.com/nymtech/nym-vpn-client/pull/5111)

### Changed
- Update the login/account creation flow for status checks (https://github.com/nymtech/nym-vpn-client/pull/5102)
- Change copy for Lewes protocol toggle (https://github.com/nymtech/nym-vpn-client/pull/5108)

### Fixed
- Fix issue with disappearing password manager dialog (https://github.com/nymtech/nym-vpn-client/pull/5102)

## [3.3.0] - 15.04.2026

### Added
- Add Lewes protocol toggle to SettingsScreen.kt (https://github.com/nymtech/nym-vpn-client/pull/5049)
- Add fallback to TimberTree when Logcat not available (https://github.com/nymtech/nym-vpn-client/pull/5049)

### Changed
- Clean up SettingsScreen.kt (https://github.com/nymtech/nym-vpn-client/pull/5028)
- Clean up SupportScreen.kt (https://github.com/nymtech/nym-vpn-client/pull/5028)
- Clean up HopScreen.kt (https://github.com/nymtech/nym-vpn-client/pull/5028)
- Update date formatting for subscription (https://github.com/nymtech/nym-vpn-client/pull/5062)

### Fixed
- Fix wording for Lewes protocol toggle (https://github.com/nymtech/nym-vpn-client/pull/5062)

## [3.2.0] - 31.03.2026

### Added
- Add Diagnostic tool (https://github.com/nymtech/nym-vpn-client/pull/4923)
- Add Lewes Protocol do Developer screen (https://github.com/nymtech/nym-vpn-client/pull/4928)
- Add autologin UI and logic (https://github.com/nymtech/nym-vpn-client/pull/4929)
- Add Random options to Entry/Exit locations list (https://github.com/nymtech/nym-vpn-client/pull/4963)
- Add info banner for subscription state (https://github.com/nymtech/nym-vpn-client/pull/4962)
- Add restricted apps update on reconnect (https://github.com/nymtech/nym-vpn-client/pull/5014)

### Changed
- Use isLinked() for Privy checks (https://github.com/nymtech/nym-vpn-client/pull/4924)

### Fixed
- Fix Crowdin config (remove video.txt) (https://github.com/nymtech/nym-vpn-client/pull/5019)

## [3.1.2] - 21.03.2026

### Changed
- Use webkpi instead of ruslts-platform-verifier and android only

## [3.1.1] - 19.03.2026

### Added
- Add Subscription and Bandwidth information (https://github.com/nymtech/nym-vpn-client/pull/4817)

### Changed
- AGP version update (https://github.com/nymtech/nym-vpn-client/pull/4586)
- Clean up QUIC feature flag usage (https://github.com/nymtech/nym-vpn-client/pull/4805)
- Welcome screen images update (https://github.com/nymtech/nym-vpn-client/pull/4814)

### Fixed
- Fix issue with the Details button for exit server (https://github.com/nymtech/nym-vpn-client/pull/4805)
- Fix issue with language list (https://github.com/nymtech/nym-vpn-client/pull/4846)

## [3.0.0] - 03.03.2026

### Added
- Add subscription checks after login (https://github.com/nymtech/nym-vpn-client/pull/4683)
- Expose account summary for handling Account state (https://github.com/nymtech/nym-vpn-client/pull/4683)
- Add Social linking check for Account Screen (https://github.com/nymtech/nym-vpn-client/pull/4714)

### Changed
- Handle login input as password (https://github.com/nymtech/nym-vpn-client/pull/4708)
- Remove "Logout" button from settings screen (https://github.com/nymtech/nym-vpn-client/pull/4708)

### Fixed
- Fix No subscription error handling on Main screen (https://github.com/nymtech/nym-vpn-client/pull/4685)
- Error handling during login (https://github.com/nymtech/nym-vpn-client/pull/4708)
- Fix Boot receiver (https://github.com/nymtech/nym-vpn-client/pull/4730)

## [2.9.0] - 18.02.2026

### Added
- Add support for account creation and registration in VpnService API (https://github.com/nymtech/nym-vpn-client/pull/4533)
- Add Mixnet tuning UI and logic (https://github.com/nymtech/nym-vpn-client/pull/4555)
- Add Privy account linking (https://github.com/nymtech/nym-vpn-client/pull/4571)
- Add Error state for Tunnel (https://github.com/nymtech/nym-vpn-client/pull/4621)

### Changed
- Refactor VPN service architecture (https://github.com/nymtech/nym-vpn-client/pull/4517)
- Implement dynamic reconnectTunnel for config updates (https://github.com/nymtech/nym-vpn-client/pull/4533)
- Optimize DnsViewModel logic (https://github.com/nymtech/nym-vpn-client/pull/4533)
- Clean up locales (https://github.com/nymtech/nym-vpn-client/pull/4568)
- Move translation download to build (https://github.com/nymtech/nym-vpn-client/pull/4568)
- Update Account Info screen UI and logic (https://github.com/nymtech/nym-vpn-client/pull/4571)
- Update notification status for Tunnel state (https://github.com/nymtech/nym-vpn-client/pull/4621)
- Update UI for Connection state handling (https://github.com/nymtech/nym-vpn-client/pull/4621)

### Fixed
- Fix fdsan crashes (https://github.com/nymtech/nym-vpn-client/pull/4533)
- Fix Always-On VPN behavior on system restart (https://github.com/nymtech/nym-vpn-client/pull/4533)
- Fix auth issue for Passphrase screen (https://github.com/nymtech/nym-vpn-client/pull/4556)
- Fix deeplink handling issue (https://github.com/nymtech/nym-vpn-client/pull/4621)

## [2.8.0]

### Added
- Add deeplinks support for Main Screen (https://github.com/nymtech/nym-vpn-client/pull/4441)
- Add logs templates for non-ui screens and modules (https://github.com/nymtech/nym-vpn-client/pull/4457)
- Add Enable Logs toggle on Logs screen (https://github.com/nymtech/nym-vpn-client/pull/4457)
- Add Enable Debug Verbose toggle on Logs screen (https://github.com/nymtech/nym-vpn-client/pull/4457)
- Add free trial line to Select Plan screen (https://github.com/nymtech/nym-vpn-client/pull/4472)

### Changed
- Add auth check for shortcuts actions (https://github.com/nymtech/nym-vpn-client/pull/4444)
- Updated format for logs export zip (folder, logs grouping) (https://github.com/nymtech/nym-vpn-client/pull/4457)
- Updated action handling for NavBar (https://github.com/nymtech/nym-vpn-client/pull/4457)
- Removed logs noise (https://github.com/nymtech/nym-vpn-client/pull/4457)

### Fixed
- Fix Auto start handling (https://github.com/nymtech/nym-vpn-client/pull/4442)
- Fix system paddings for Select Plan screen (https://github.com/nymtech/nym-vpn-client/pull/4472)

## [2.7.0] - 2026.01.16

### Added
- Locale for high/low copy on Details screen (https://github.com/nymtech/nym-vpn-client/pull/4274)
- Add reconnect on LAN bypass (https://github.com/nymtech/nym-vpn-client/pull/4282)
- Add Onboarding flow (https://github.com/nymtech/nym-vpn-client/pull/4322)
- Add technical opt screen after login/purchase (https://github.com/nymtech/nym-vpn-client/pull/4322)

### Changed
- Replace mnemonic and access code with passphrase (https://github.com/nymtech/nym-vpn-client/pull/4274)
- Replace gateway with server (https://github.com/nymtech/nym-vpn-client/pull/4274)
- Grey-out AmneziaWG and Stealth API toggles on Censorship screen (https://github.com/nymtech/nym-vpn-client/pull/4277)
- Change sections order on Server Details screen (https://github.com/nymtech/nym-vpn-client/pull/4277)
- Disable Custom DNS toggle if DNS list is empty (https://github.com/nymtech/nym-vpn-client/pull/4277)
- Updates for Split tunneling screen: reconnect and save changes logic (https://github.com/nymtech/nym-vpn-client/pull/4282)
- Changes for Custom DNS screen reconnecting logic (https://github.com/nymtech/nym-vpn-client/pull/4282)
- Remove reconnect modals for QUIC on Censorship screen, add snack bar and debounce (https://github.com/nymtech/nym-vpn-client/pull/4282)
- Promote VPN to foreground immediately at service entry (https://github.com/nymtech/nym-vpn-client/pull/4291)
- UI changes for User welcome screen (https://github.com/nymtech/nym-vpn-client/pull/4322)
- Play Store visual updates (https://github.com/nymtech/nym-vpn-client/pull/4363)

### Fixed
- "Add" button on DNS screen now expand to the left for verbose languages (https://github.com/nymtech/nym-vpn-client/pull/4277)
- Censorship screen blank on Chinese phones (https://github.com/nymtech/nym-vpn-client/pull/4277)
- Clear notifications after successful connection (https://github.com/nymtech/nym-vpn-client/pull/4327)
- Clear notifications when another VPN started (https://github.com/nymtech/nym-vpn-client/pull/4327)
- Bypass traffic to local DNS server when LAN bypass is disabled. Align LAN bypass rules with desktop (https://github.com/nymtech/nym-vpn-client/pull/4366)

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
