# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]

## [2026.12.2-beta.1] - 2026-08-23

## [2026.12.1] - 2026-08-21

### Changed

- Local DNS resolver will respond with `serv_fail` on timeout from upstream DNS server (https://github.com/nymtech/nym-vpn-client/pull/6132)
- Remove IPv6 DNS addresses from default DNS configuration due to reliability issues (https://github.com/nymtech/nym-vpn-client/pull/6132)


## [2026.12.0] - 2026-08-18

### Added

- [CLI] Add short option `-l` for `nym-vpnc status --listen` (https://github.com/nymtech/nym-vpn-client/pull/5839)
- Recents manager for storing successful gateway connections (https://github.com/nymtech/nym-vpn-client/pull/5903)
- Favorites manager for storing UI favorites (https://github.com/nymtech/nym-vpn-client/pull/5914)
- Geo-Exclusion now supports Russia (https://github.com/nymtech/nym-vpn-client/pull/5917)
- If the host doesn't have an IPv6 address then split tunnelling is disabled for IPv6 (https://github.com/nymtech/nym-vpn-client/pull/6052) 

### Changed

- While in Connected state swap internal resolver to use custom DNS (via system resolver). (https://github.com/nymtech/nym-vpn-client/pull/5674)
- Use "Auto" for entry and exit selectors independently (https://github.com/nymtech/nym-vpn-client/pull/5962)
- QUIC bridges wait 21s for the first WireGuard packet (was 10s).

### Fixed

- Ad-blocker and nym-socks5-proxy files are no longer stored in the network directory. (https://github.com/nymtech/nym-vpn-client/pull/5826)
- Improve behavior of forwarding resolver by not sending empty response when hostname resolution fails. Instead simulate timeout to let clients retry more aggressively. (https://github.com/nymtech/nym-vpn-client/pull/5832)
- When no VPN tunnel is active the geo-exclusion feature rejects non-excluded traffic. (https://github.com/nymtech/nym-vpn-client/pull/5872)


## [2026.11.0] - 2026-07-10

### Added

- [iOS] Send network stats over the tunnel interface (https://github.com/nymtech/nym-vpn-client/pull/5564)
- Store gateway independence notification toggle (https://github.com/nymtech/nym-vpn-client/pull/5586)

### Changed

- Permit API networking in error state in order to refresh account data (https://github.com/nymtech/nym-vpn-client/pull/5623)
- Increase timeout for TCP-based probe for connection monitoring in two-hop mode (https://github.com/nymtech/nym-vpn-client/pull/5803)

### Fixed

- Fix missing "recursion available" flag in DNS responses, passthrough authority and additional records (https://github.com/nymtech/nym-vpn-client/pull/5546)
- Provide escape hatch when reconnecting the tunnel in "time desynced" error state (https://github.com/nymtech/nym-vpn-client/pull/5551)
- Race when network usage spikes happen during longer bandwidth checks, disconnecting the client from the server (https://github.com/nymtech/nym-vpn-client/pull/5618)
- [macOS] Disable authentication when flag is set (https://github.com/nymtech/nym-vpn-client/pull/5645)
- [iOS] Fix metadata endpoint not being reached for exit tunnel (https://github.com/nymtech/nym-vpn-client/pull/5728)
- Fix going into Connected state when metadata endpoint might not work (https://github.com/nymtech/nym-vpn-client/pull/5750)
- [Linux] Disable NetworkManager's connectivity check before applying firewall rules (https://github.com/nymtech/nym-vpn-client/pull/5801) 
- [iOS] Skip ad-blocking rules that do not block by domain (https://github.com/nymtech/nym-vpn-client/pull/5658)
- [iOS] Handle sub-domain blocking (https://github.com/nymtech/nym-vpn-client/pull/5810)
- [macOS] Skip catch-all NAT masquerade when split tunneling is active (macOS >=14.6, <15.1 only). (https://github.com/nymtech/nym-vpn-client/pull/5569)
- [Android] Fix old Android devices failing to bind for metadata endpoint (https://github.com/nymtech/nym-vpn-client/pull/5878)

### Removed

- Removed mixnet tuning feature flag (https://github.com/nymtech/nym-vpn-client/pull/5581)


## [2026.10.0] - 2026-06-09

### Added

- Node family restriction and possibility to check for probable gateway selection before connection (https://github.com/nymtech/nym-vpn-client/pull/5285)
- Enable detection for bad gateways even when specifically selected (https://github.com/nymtech/nym-vpn-client/pull/5429)
- Add better exit gateway wireguard handshake checking to mitigate ICMP ping failures (https://github.com/nymtech/nym-vpn-client/pull/5481)
- Gateway subnet check included in subnet independence criteria (https://github.com/nymtech/nym-vpn-client/pull/5484)
- [Linux] Add fallback for polkit policy path (https://github.com/nymtech/nym-vpn-client/pull/5528)

### Changed

- Change default entry and exit points to random (https://github.com/nymtech/nym-vpn-client/pull/5378)
- Enable secure DNS for requests forwarded by local resolver (https://github.com/nymtech/nym-vpn-client/pull/5458)

### Fixed

- [Android] Diagnostic doesn't panic because of uninitialized context (https://github.com/nymtech/nym-vpn-client/pull/5415)
- [Windows] Fix a crash when the network reconnected (https://github.com/nymtech/nym-vpn-client/pull/5508)
- [Linux] LP firewalled by allowed_endpoints (https://github.com/nymtech/nym-vpn-client/pull/5516)


## [1.30] - 2026-05-29

### Added

- [iOS] Introduce ad-blocker. (https://github.com/nymtech/nym-vpn-client/pull/5227)

### Fixed

- Fix panic when restoring default routes (https://github.com/nymtech/nym-vpn-client/pull/5225)
- Fix adblocker deactivation caused by remote returning embedded HTTP errors (https://github.com/nymtech/nym-vpn-client/pull/5302)
- Don't reuse entry gateway when registering fails (https://github.com/nymtech/nym-vpn-client/pull/5379)
- [macOS] Daemon checks against the correct ID for its own signature (https://github.com/nymtech/nym-vpn-client/pull/5390)

### Changed

- Disable automatic gateway elections and revert back to hard-coded `Explicit` mode (https://github.com/nymtech/nym-vpn-client/pull/5436)


## [1.29.2] - 2026-05-04

### Fixed

- [Windows] Fix missing IPv4 on mixnet tunnel adapter (https://github.com/nymtech/nym-vpn-client/pull/5206)


## [1.29.1] - 2026-04-29

### Changed

- Switch platform to patched `2026.7-tola`


## [1.29.0] - 2026-04-29

### Added

- Quick connect algorithm (https://github.com/nymtech/nym-vpn-client/pull/5112)
- Add TCP listener for local DNS resolver (https://github.com/nymtech/nym-vpn-client/pull/5113)
- Add SOCKS5 Proxy process to implement Geo Exclusion (https://github.com/nymtech/nym-vpn-client/pull/5078)
- Disable client verifications on daemon flag for debug purposes (https://github.com/nymtech/nym-vpn-client/pull/5148)
- [Android] Add Geo Exclusion support via SOCKS5 Proxy (https://github.com/nymtech/nym-vpn-client/pull/5160)
- Propagate `fairUsage.dataUnavailable` from API through to clients so a database outage no longer surfaces as a bandwidth-exceeded error (https://github.com/nymtech/nym-vpn-client/pull/5217)

### Changed

- [macOS] Use endpoint-security framework directly instead of parsing eslogger output (https://github.com/nymtech/nym-vpn-client/pull/4749)

### Fixed


- Fix false bandwidth-exceeded errors when the VPN API fair-usage database is temporarily unavailable (https://github.com/nymtech/nym-vpn-client/pull/5217)
- Fix accounts incorrectly appearing inactive due to malformed API timestamp fields (https://github.com/nymtech/nym-vpn-client/pull/5217)
- [iOS/macOS] Fix account summary fetch errors being silently swallowed, leaving the UI in an unresponsive state (https://github.com/nymtech/nym-vpn-client/pull/5217)
- Unify `VpnAccountSummary` timestamp parsing through a single `parse_timestamp` helper that warns on malformed input. Only `fair_usage.resetsOnUtc` soft-fails to `None`; subscription and auth-method timestamps now propagate `PayloadError` so a bad payload fails loudly instead of silently flipping subscriptions to inactive (root cause of NYM-1156 "Requesting ZkNyms" / "Get Started" hangs on v2.22.0 iOS).
- [iOS/macOS] Stop swallowing errors from `fetchAccountSummary` with `try?`; log a sanitized line (error type only, no raw payload string) and set `accountSummaryLastFetchFailed` so the UI can observe failure without parsing device logs.
- [Linux] Add Polkit as deb and arch dependency (https://github.com/nymtech/nym-vpn-client/pull/5143)

## [1.28.0] - 2026-04-14

### Added

- [Windows] App Split Tunnelling (https://github.com/nymtech/nym-vpn-client/pull/4908).
- CLI: add command to list processes excluded from VPN tunnel: `nym-vpnc split-tunnel excluded-processes` (https://github.com/nymtech/nym-vpn-client/pull/4905)
- [Linux] Add support for per-app split-tunneling (https://github.com/nymtech/nym-vpn-client/pull/5001).

### Changed

- Stream SelectedGateways via buffered selection (https://github.com/nymtech/nym-vpn-client/pull/5037)

### Fixed

- [macOS] Fix bug in XPC buffering between XPC and gRPC layers (https://github.com/nymtech/nym-vpn-client/pull/4985)


## [1.27.0] - 2026-03-31

- [macOS] XPC as transport layer between clients and daemon (https://github.com/nymtech/nym-vpn-client/pull/4695)
- [macOS] Authentication layer for windows, feature gated (https://github.com/nymtech/nym-vpn-client/pull/4802)
- Activate authentication layer on all desktop platforms (https://github.com/nymtech/nym-vpn-client/pull/4856)

### Fixed

- [macOS] XPC client stall when daemon is not running (https://github.com/nymtech/nym-vpn-client/pull/4973)


## [1.26.0] - 2026-03-17

### Added

- [CLI] `nym-vpnc account set` now uses `--location blockchain`; aliases keep legacy `--mode decentralised` and `--mode decentralized` working.
- [CLI] `nym-vpnc account obtain-ticketbooks` subcommand renamed (legacy alias `decentralised-obtain-ticketbooks` still works). `--source` (currently parsed but all sources route to smartcontract backend).


## [1.25.0] - 2026-03-02

### Added

- [Windows] Authentication layer for windows, still feature gated (https://github.com/nymtech/nym-vpn-client/pull/4618)
- [macOS] Add support for per-app split-tunneling (https://github.com/nymtech/nym-vpn-client/pull/4694)

### Fixed

- Detect time travel and sleep when obtaining remote time (https://github.com/nymtech/nym-vpn-client/pull/4604)


## [1.24.0] - 2026-02-12

### Added

- Added privy UI feature flag (https://github.com/nymtech/nym-vpn-client/pull/4223)
- Added TraceID and SpanID for the account controller commands (https://github.com/nymtech/nym-vpn-client/pull/4426)
- Added mixnet tuning feature flag (https://github.com/nymtech/nym-vpn-client/pull/4514)
- [Linux] Password-based authentication for clients that attempt to connect to daemon; feature gated until front-end is implemented (https://github.com/nymtech/nym-vpn-client/pull/4538)

### Changed

- Changed VPN API HTTP timeout from 60s to 30s. (https://github.com/nymtech/nym-vpn-client/pull/4604)
-

### Fixed

- Fix discovery propagation bug (https://github.com/nymtech/nym-vpn-client/pull/4226)
- Ensure that vpn topology is refreshed periodically when connecting (https://github.com/nymtech/nym-vpn-client/pull/4228)
- [Android] Bypass local DNS servers (https://github.com/nymtech/nym-vpn-client/pull/4347)
- Fix gateway cache and topology cache not being invalidated when remote discovery updates are received. Note: Manual environment switching still requires daemon/app restart (https://github.com/nymtech/nym-vpn-client/pull/4464)

### Removed

- Removed credentials mode feature flag from code base (https://github.com/nymtech/nym-vpn-client/pull/4223)

### Changed

- [Android] Enable debug logs in production builds for core library (https://github.com/nymtech/nym-vpn-client/pull/4405)
- [Android] Print library logs to file, in addition to the existing logcat (https://github.com/nymtech/nym-vpn-client/pull/4432)


## [1.21.0] - 2025-12-15

### Added

- Add custom DNS setting for mobile platforms (https://github.com/nymtech/nym-vpn-client/pull/4106)
- Login with signature string in addition to mnemonic (https://github.com/nymtech/nym-vpn-client/pull/4117)
- SOCKS5 proxy can now be controlled via `nym-vpnc` (https://github.com/nymtech/nym-vpn-client/pull/4148)

### Fixed

- Increase the number of Windows firewall slots (https://github.com/nymtech/nym-vpn-client/pull/4072)
- Enable two-hop by default (https://github.com/nymtech/nym-vpn-client/pull/4090)

### Changed

- Update default entry and exit points to Switzerland (https://github.com/nymtech/nym-vpn-client/pull/XXX)

### Removed

- CLI: remove legacy call to connect the tunnel (https://github.com/nymtech/nym-vpn-client/pull/4094)


## [1.20.0] - 2025-12-01

### Added

- Custom DNS servers can be used, instead of the pre-defined ones. They can be set and cleared using the CLI `nym-vpnc dns` command (https://github.com/nymtech/nym-vpn-client/pull/4015)

### Changed

- Rotate wireguard keys every 1-2 weeks, if disconnected (https://github.com/nymtech/nym-vpn-client/pull/3788)
- When querying for bandwidth, retry once on failure (https://github.com/nymtech/nym-vpn-client/pull/3922).

### Fixed

- Avoid connection looping by temporarily blacklisting the entry gateway (https://github.com/nymtech/nym-vpn-client/pull/4047)


## [1.19.0] - 2025-11-19

### Added

- Implement a TCP-based probe as a fallback for connection monitoring when ICMP is unavailable. (https://github.com/nymtech/nym-vpn-client/pull/3868)
- Expose A/C's `RequestingZkNyms` state to UI for in app payment flows (https://github.com/nymtech/nym-vpn-client/pull/3925)

### Changed

- Rotate wireguard keys every 1-2 weeks, if disconnected (https://github.com/nymtech/nym-vpn-client/pull/3788)
- When querying for bandwidth, retry once on failure (https://github.com/nymtech/nym-vpn-client/pull/3922).

### Fixed

- [macOS] Prevent resetting state for non-tunnel DNS connections (https://github.com/nymtech/nym-vpn-client/pull/3899)
- Filter out gateways that might be blacklisted by mixnet (https://github.com/nymtech/nym-vpn-client/pull/3948)

### Removed

- Remove unnecessary DNS resolutions on mobile platforms where there is no configurable firewall. (https://github.com/nymtech/nym-vpn-client/pull/3913)


## [1.18.0] - 2025-11-03

### Added

- Add new CLI commands to manage sentry and anonymous network statistics collection (https://github.com/nymtech/nym-vpn-client/pull/3695)
- Add tunnel connection monitoring (https://github.com/nymtech/nym-vpn-client/pull/3724)
- Backend QUIC filtering for desktop (https://github.com/nymtech/nym-vpn-client/pull/3746)
- Fallback on mixnet channel if metadata endpoint is not available (https://github.com/nymtech/nym-vpn-client/pull/3747)
- Library exposing the command for manual wireguard key rotation (https://github.com/nymtech/nym-vpn-client/pull/3870)

### Changed

- Use two keypairs (entry & exit) per gateway (https://github.com/nymtech/nym-vpn-client/pull/3591)
- Disable system DNS resolver fallback on primary resolver failure (https://github.com/nymtech/nym-vpn-client/pull/3832)

### Fixed

- Fix mixnet listener timeout not being set (https://github.com/nymtech/nym-vpn-client/pull/3715)
- Prevent account controller from networking while state machine is in offline state (https://github.com/nymtech/nym-vpn-client/pull/3723)
- [macOS] Log error instead of failing when removing keys from dynamic store during DNS reset. (https://github.com/nymtech/nym-vpn-client/pull/3711)
- CLI: fix hang when calling `nym-vpnc disconnect --wait` in disconnected state. (https://github.com/nymtech/nym-vpn-client/pull/3743)
- Don't log a warning on some expected value from the API (https://github.com/nymtech/nym-vpn-client/pull/3763)
- Fix no gateway id problem (https://github.com/nymtech/nym-vpn-client/pull/3768)
- [Windows] Wait for network interface addresses become usable before starting the tunnel (https://github.com/nymtech/nym-vpn-client/pull/3773)
- Fix network environment updates not being made available for grpc clients (https://github.com/nymtech/nym-vpn-client/pull/3805)
- Ensure that default discovery when written to disk is always considered stale (https://github.com/nymtech/nym-vpn-client/pull/3805)
- Make discovery refresh aware of network connectivity (https://github.com/nymtech/nym-vpn-client/pull/3805)
- Fix database cleanup when forgetting account (https://github.com/nymtech/nym-vpn-client/pull/3825)


## [1.17.0] - 2025-10-17

### Added

- Get more gateway details, parse them, and expose them to UI to be shown in the server details page (https://github.com/nymtech/nym-vpn-client/pull/3447)
- Allow for random selection inside a US state (https://github.com/nymtech/nym-vpn-client/pull/3489)
- Add control over LAN sharing when device connection is secured (https://github.com/nymtech/nym-vpn-client/pull/3496)
- The
  `nym-vpnc status --listen` command now prints the daemon configuration when it's changed by other clients (https://github.com/nymtech/nym-vpn-client/pull/3503).
- Users can select residential only exit nodes (https://github.com/nymtech/nym-vpn-client/pull/3560).

### Changed

- LAN sharing is off by default. Use "Allow LAN" setting to allow it (https://github.com/nymtech/nym-vpn-client/pull/3496)
- Differentiate between entry and exit gateway errors (https://github.com/nymtech/nym-vpn-client/pull/3458)
- New CLI command interface. Legacy commands will continue working until the following release. (https://github.com/nymtech/nym-vpn-client/pull/3559)

### Fixed

- Don't retry on disappeared entry or exit gateway and return to UI for selecting again (https://github.com/nymtech/nym-vpn-client/pull/3520)
- Recover from error loop when mixnet client can't reach gateway after a number of retries (https://github.com/nymtech/nym-vpn-client/pull/3694)

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
- [macOS] Fix issues with DNS not being properly reset on disconnect on macOS 15. (https://github.com/nymtech/nym-vpn-client/pull/3232)
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
