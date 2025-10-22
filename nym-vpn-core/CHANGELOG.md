# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Use two keypairs (entry & exit) per gateway (https://github.com/nymtech/nym-vpn-client/pull/3591)


## [1.17.0] - 2025-10-17

### Added

- Get more gateway details, parse them, and expose them to UI to be shown in the server details page (https://github.com/nymtech/nym-vpn-client/pull/3447)
- Allow for random selection inside a US state (https://github.com/nymtech/nym-vpn-client/pull/3489)
- Add control over LAN sharing when device connection is secured (https://github.com/nymtech/nym-vpn-client/pull/3496)
- The `nym-vpnc status --listen` command now prints the daemon configuration when it's changed by other clients (https://github.com/nymtech/nym-vpn-client/pull/3503).
- Users can select residential only exit nodes (https://github.com/nymtech/nym-vpn-client/pull/3560).

### Changed

- LAN sharing is off by default. Use "Allow LAN" setting to allow it (https://github.com/nymtech/nym-vpn-client/pull/3496)
- Differentiate between entry and exit gateway errors (https://github.com/nymtech/nym-vpn-client/pull/3458)
- New CLI command interface. Legacy commands will continue working until the following release. (https://github.com/nymtech/nym-vpn-client/pull/3559)

### Fixed

- Don't retry on disappeared entry or exit gateway and return to UI for selecting again (https://github.com/nymtech/nym-vpn-client/pull/3520)
- Recover from error loop when mixnet client can't reach gateway after a number of retries (https://github.com/nymtech/nym-vpn-client/pull/3694)
- Fix MacOS load/remove bug in store (https://github.com/nymtech/nym-vpn-client/pull/3711)

### Removed

- Removed countries query (https://github.com/nymtech/nym-vpn-client/pull/3523)


## [1.16.0] - 2025-09-26

### Added

- Expose exit IPs (v4 and v6) as well as gateway version from the core (https://github.com/nymtech/nym-vpn-client/pull/3427)

### Fixed

- Fix edge case where mixnet processor could be blocked from exiting by mixnet listener causing the client to be stuck in disconnecting state (https://github.com/nymtech/nym-vpn-client/pull/3394)
- Fix Sentry extra metadata tag when there is no OS extra info (https://github.com/nymtech/nym-vpn-client/pull/3411)

### Changed

- [macOS] Skip filtering loopback traffic to optimize performance (https://github.com/nymtech/nym-vpn-client/pull/3441)
- Prioritize high performance gateways first, fallback to medium. This rule does not apply when specific gateway is selected explicitly (https://github.com/nymtech/nym-vpn-client/pull/3511)


## [1.15.0] - 2025-09-10

### Added

- Provide metadata to keep track of progress when establishing connection (https://github.com/nymtech/nym-vpn-client/pull/3351)

### Fixed

- [Windows] Embed core version into `winfw.dll` and `libwg.dll` (https://github.com/nymtech/nym-vpn-client/pull/3292)
- Disable mixnet cover traffic in two-hop mode (https://github.com/nymtech/nym-vpn-client/pull/3347)
- Prevent discovery file from becoming stale because it's only refreshed whilst connected (https://github.com/nymtech/nym-vpn-client/pull/3377)

### Changed

- Daemon global and service configuration is now stored in JSON format, allowing versioning to be
  supported (https://github.com/nymtech/nym-vpn-client/pull/3344).
- Use intra-tunnel endpoint for querying and topping up bandwidth, replacing the mixnet channel (https://github.com/nymtech/nym-vpn-client/pull/3316)

## [1.14.0] - 2025-08-26

### Added

- Introduce more extensive entry/exit country parsing in
  nym-vpn-cli (https://github.com/nymtech/nym-vpn-client/pull/3235)

### Changed

- Upgrade Nym platform to emmental release (https://github.com/nymtech/nym-vpn-client/pull/3155)
- Enable anonymous network statistics collection by default in the daemon, only for new
  installations (https://github.com/nymtech/nym-vpn-client/pull/3265)
- Reconnect on failure to resolve gateway addresses instead of entering error
  state (https://github.com/nymtech/nym-vpn-client/pull/3268)
- Reconnect to new gateways every 2 failed connection attempts (https://github.com/nymtech/nym-vpn-client/pull/3273)

### Fixed

- Improve shutdown sequence by exiting internal components in the reverse order of their creation. Drain tunnel events
  and deliver them to listeners before exiting the daemon. (https://github.com/nymtech/nym-vpn-client/pull/3185)
- Fix potential infinite loop when sending a disconnect message over mixnet. Limit disconnect timeout to 5 seconds and
  add 500ms delay between retries. (https://github.com/nymtech/nym-vpn-client/pull/3160)
- Prevent gateways refresh from blocking daemon shutdown during
  initialization. (https://github.com/nymtech/nym-vpn-client/pull/3160)
- Add timeout to DNS resolution fixing indefinite connecting
  state. (https://github.com/nymtech/nym-vpn-client/pull/3231)
- [macOS] Fix issues with DNS not being properly reset on disconnect on macOS
    15. (https://github.com/nymtech/nym-vpn-client/pull/3232)
- [macOS] Bind DNS resolver to random loopback IP on port 53 to fix compatibility issues with other software, notably
  `dig` and `nslookup`. (https://github.com/nymtech/nym-vpn-client/pull/3232)

## [1.13.1] - 2025-07-30

### Changed

- Update pre-bundled discovery to include account links (https://github.com/nymtech/nym-vpn-client/pull/3167)
- Reduce noisiness of WireGuard logs (https://github.com/nymtech/nym-vpn-client/pull/3169)

## [1.13.0] - 2025-07-29

### Added

- Add setting to toggle IPv6 support.
- vpnd: Add support to toggle network statistics collection.

### Fixed

- Box too large futures to fix stackoverflow on Windows (https://github.com/nymtech/nym-vpn-client/pull/3139)

## [1.12.0] - 2025-07-18

### Added

- Register with locally generated mnemonic (https://github.com/nymtech/nym-vpn-client/pull/2926)
- Probe sends zk-nyms (https://github.com/nymtech/nym-vpn-client/pull/3011)
- Two keypairs per gateway (first part) (https://github.com/nymtech/nym-vpn-client/pull/3035)
- Don't wait on topology fetch from network on state machine start (https://github.com/nymtech/nym-vpn-client/pull/3072)

### Changed

- Use nym cheddar fork (https://github.com/nymtech/nym-vpn-client/pull/3048)

### Removed

- Remove a shutdown timeout for tonic server (https://github.com/nymtech/nym-vpn-client/pull/2938)
- Remove shared mixnet client (https://github.com/nymtech/nym-vpn-client/pull/2967)
- Remove wireguard credential mode flag (https://github.com/nymtech/nym-vpn-client/pull/3021)

### Fixed

- Fix bug that prevented the database(s) from closing gracefully before being
  disposed (https://github.com/nymtech/nym-vpn-client/pull/2925)
- Unblock mixnet client because of a deadlock (https://github.com/nymtech/nym-vpn-client/pull/3039)
- Apply patch to h2 crate so hickory-dns DoH connections consider server go-away close as valid preventing spurious warn
  logging (https://github.com/nymtech/nym-vpn-client/pull/3053)
- Fix task manager dropping immediately on config path not being
  specified (https://github.com/nymtech/nym-vpn-client/pull/3054)
- Fix tunnel connectivity issues by applying route MTU for multihop
  tunnel (https://github.com/nymtech/nym-vpn-client/pull/3051)
- Fix prefetching topology not working at no network daemon boot (https://github.com/nymtech/nym-vpn-client/pull/3072)

## [1.11.0] - 2025-06-18

### Fixed

- Fix persistent mixnet storage failure preventing the client from starting
- Fix issues preventing the daemon from starting without network connectivity
- [macOS] Improve route monitoring and offline detection
