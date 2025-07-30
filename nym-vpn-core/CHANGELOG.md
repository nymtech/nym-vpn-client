# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

- Fix bug that prevented the database(s) from closing gracefully before being disposed (https://github.com/nymtech/nym-vpn-client/pull/2925)
- Unblock mixnet client because of a deadlock (https://github.com/nymtech/nym-vpn-client/pull/3039)
- Apply patch to h2 crate so hickory-dns DoH connections consider server go-away close as valid preventing spurious warn logging (https://github.com/nymtech/nym-vpn-client/pull/3053)
- Fix task manager dropping immediately on config path not being specified (https://github.com/nymtech/nym-vpn-client/pull/3054)
- Fix tunnel connectivity issues by applying route MTU for multihop tunnel (https://github.com/nymtech/nym-vpn-client/pull/3051)
- Fix prefetching topology not working at no network daemon boot (https://github.com/nymtech/nym-vpn-client/pull/3072)


## [1.11.0] - 2025-06-18

### Fixed

- Fix persistent mixnet storage failure preventing the client from starting
- Fix issues preventing the daemon from starting without network connectivity
- [macOS] Improve route monitoring and offline detection
