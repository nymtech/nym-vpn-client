# Android: split tunneling under always-on VPN lockdown

**Date:** 2026-08-07
**Status:** Draft — pending review
**Scope:** nym-vpn-android + nym-vpn-core (Rust) + wireguard/libwg (Go)

## Problem

When Android's "Always-on VPN" + "Block connections without VPN" (lockdown) is enabled — the default on GrapheneOS — apps excluded from the tunnel via split tunneling lose all connectivity. This is intended AOSP behavior: excluded apps (`VpnService.Builder.addDisallowedApplication()`) use system networking "as if the VPN wasn't running", which is exactly what lockdown blocks. There is no app-side exemption: `setAllowBypass()` is neutralized under lockdown (`bypassable = allowBypass && !mLockdown` in AOSP `Vpn.java`), and the OS lockdown allowlist is settable only by an enterprise Device Policy Controller.

We want excluded apps to keep working without asking users to disable lockdown.

## Prior art

- Mullvad, official WireGuard, ProtonVPN, Tailscale, Windscribe, shadowsocks-android: all document the limitation or wontfix it. None work around it.
- Clash Meta for Android, sing-box, RethinkDNS, NetGuard (filtering mode): solve it with **in-tunnel per-UID routing** — no `addDisallowedApplication`; all apps route into the TUN (satisfying lockdown), a userspace stack attributes each new flow to an app via `ConnectivityManager.getConnectionOwnerUid()` (API 29+), and flows belonging to excluded apps are forwarded directly over sockets the VPN process opens and marks with `VpnService.protect()`. This works because AOSP unconditionally exempts the VPN app's own UID from lockdown.

## Goals

1. Excluded apps get direct internet access while lockdown is enabled, on Android 10+ (API 29+), in both WireGuard (2-hop) and mixnet (5-hop) modes.
2. Zero behavior/overhead change when lockdown is off, when the exclusion list is empty, or on API 24–28.
3. Fail-closed: no attribution failure may ever cause *tunneled* traffic to leak outside the tunnel.

## Non-goals

- ICMP support for excluded apps (cannot be attributed to a UID; ping from excluded apps will not work).
- Per-app distinction within a shared UID (Android limitation).
- Changing desktop split tunneling (`nym-split-tunnel` is desktop-only and untouched).
- Lobbying Google/GrapheneOS for OS-level exemptions (worth doing, but independent of this work).

## Design overview

Two strategies, selected at TUN-establish time in `VpnTunController.configureTunnel()`:

| Condition | Strategy |
|---|---|
| No excluded apps | Current behavior, no change |
| Excluded apps, lockdown **off** (or API < 29) | Current behavior: `addDisallowedApplication()` (kernel routing, zero overhead, excluded traffic never touches our process) |
| Excluded apps, lockdown **on** (API 29+, `isLockdownEnabled()`) | **Flow steering:** no `addDisallowedApplication`; a steering layer owns the TUN fd and forwards excluded apps' flows directly over protected sockets |

On API 24–28 lockdown cannot be detected (`isLockdownEnabled()` is API 29); the split-tunneling screen shows a static informational note that lockdown will block excluded apps (see UI section).

### The steering layer (Go, in `wireguard/libwg`)

A new Go component, following the pattern of the existing `dns_filter_proxy.rs` (socketpair-as-fake-TUN) and reusing `libwg`'s existing gVisor netstack and socket↔netstack forwarders (`forwarders/tcp.go`, `forwarders/udp.go`).

**Placement:** Rust creates an `AF_UNIX SOCK_DGRAM` socketpair. The steering layer gets the real TUN fd plus one end of the pair; the existing downstream consumer — wireguard-go (optionally behind the DNS filter proxy, which chains with the same shim pattern) or the Rust `MixnetProcessor` — gets the other end and is otherwise unchanged. Go is chosen over Rust because gVisor and the forwarders already live in `libwg`, and `libwg` is linked into the Android build in both modes.

**Packet path (upstream, from apps):**
1. Read raw IP packet from the real TUN fd.
2. Look up the flow (5-tuple) in a flow table.
3. **New flow:** classify — call the attribution callback (below) → UID → in excluded-UID set?
   - *Excluded:* mark flow BYPASS; inject packet into a dedicated gVisor netstack instance. TCP is terminated there and bridged to a real socket dialed on the underlying network; UDP flows get a NAT-style relay socket. Every outbound socket fd is passed through the existing bypass callback (`AndroidTunProvider.bypass(fd)` → `VpnService.protect()`) before connect.
   - *Tunneled, unattributable (`INVALID_UID`), or lookup error:* mark flow TUNNEL (fail-closed) and write the packet to the socketpair.
4. **Known flow:** route per the cached mark. No per-packet attribution calls.

**Packet path (downstream, to apps):** merge packets from the socketpair (tunnel) and the bypass netstack, write to the real TUN fd.

**Efficiency:** tunneled traffic pays one extra fd copy — the same cost the shipping DNS-filter (ad-block) proxy already imposes on the WG path. Userspace TCP termination applies only to excluded apps' flows.

### Attribution and configuration plumbing

The excluded-app list currently lives only in Kotlin and is applied only via the builder. New plumbing:

- **Excluded UIDs → core:** Kotlin resolves package names to UIDs (`PackageManager.getApplicationInfo().uid`) at connect/reconnect time and sends the UID set plus a `steeringEnabled` flag to Rust via a new `NymVpnServiceCommandSender` setter, alongside the existing ones dispatched in `VpnCoreController.applyConfigDiffToSender()`. Rust stores it next to the other per-connection options and threads it into `TunnelSettings` (`nym-vpn-lib/src/tunnel_provider/mod.rs`) and from there into the steering layer's config.
- **Connection-owner callback:** extend the `AndroidTunProvider` uniffi trait (`nym-vpn-lib-uniffi/src/tunnel_provider/android.rs`) with `get_connection_owner_uid(protocol: i32, source: SocketAddr, dest: SocketAddr) -> i32`, implemented in Kotlin via `ConnectivityManager.getConnectionOwnerUid()`. Called once per new flow from Go via the existing Rust↔Go boundary; result cached in the flow table. Clash Meta uses the identical call pattern without rate-limit issues.

### DNS for excluded apps

Excluded apps' DNS queries arrive in the TUN addressed to the tunnel's DNS server. If forwarded through the tunnel, "direct" apps would still resolve through Nym (functional and privacy mismatch, and broken if the tunnel is down while lockdown holds). Instead: Kotlin obtains the underlying (non-VPN) network's resolvers via `ConnectivityManager` link properties and passes them in the steering config; the steering layer rewrites excluded flows' DNS destination to those resolvers and relays over protected sockets. If no underlying resolvers are known, excluded apps' DNS falls back to the tunnel path (name resolution keeps working; only the resolver path is non-direct in that degraded case).

### IPv6

The TUN already claims IPv6 routes (`compute_tunnel_networks`). Excluded v6 flows are forwarded via protected v6 sockets when the underlying network has IPv6; otherwise the dial fails and the app's Happy Eyeballs falls back to v4. No NAT64.

### Error handling

- Attribution `SecurityException` / `INVALID_UID` / callback failure → flow goes through the tunnel. Never bypass on uncertainty.
- Bypass dial failure → TCP RST / ICMP port-unreachable synthesized back to the app (netstack default), so apps fail fast instead of hanging.
- Steering-layer fatal error → tear down the tunnel through the normal error path (fail-closed; lockdown then blocks everything, which is the user's chosen posture).
- Flow table: LRU + idle timeouts (TCP by state, UDP ~60 s) to bound memory.

### UI

- Split-tunneling screen, lockdown active (API 29+): informational note that excluded apps connect directly outside the tunnel and that ping/ICMP won't work for them.
- Split-tunneling screen, API 24–28 with exclusions: static warning that Android's "Block connections without VPN" setting will block excluded apps, with a deep link to always-on VPN settings.
- Privacy note in the existing `SplitTunnelingInfoModal`: under lockdown, excluded traffic transits the Nym VPN process (still direct to the internet from the device's real IP, never through the tunnel).

### Targeted fix included in scope

The core's persisted copy of the exclusion list (`KEY_RESTRICTED_APPS`) is refreshed only in `startTunnel()`/`requestReconnect()` (`ServiceBackedBackendManager.kt:113-119,162-174`) — not on the always-on boot path (`VpnService.kt:190-195`), which silently uses the last persisted value. Since always-on is precisely this feature's target scenario, the boot path must re-read the app-side list (or the two stores must be unified) so exclusions are correct on boot-started tunnels.

## Testing

- **Go unit tests:** flow-table classification (new/known flows, INVALID_UID → tunnel, LRU/timeout), packet merge ordering, DNS rewrite.
- **Kotlin unit tests:** strategy selection matrix (list empty / lockdown on / off / API level), package→UID resolution, underlying-resolver lookup.
- **Manual matrix (must pass before release):** {lockdown on, off} × {WG mode, mixnet mode} × excluded app doing {TCP (browse), UDP (QUIC), DNS} + tunneled app unaffected + after tunnel teardown under lockdown, all apps (including previously excluded ones) are blocked by the OS. Verify on stock Android 14+ and GrapheneOS. Confirm ad-block (DNS filter proxy) still works when chained with steering.
- **Leak check:** with steering active, packet capture on the underlying network shows only (a) tunnel-endpoint traffic and (b) excluded apps' flows — nothing else.
- Maestro flow update for the split-tunneling screen states.

## Risks / open questions

- `getConnectionOwnerUid` misses sockets bound with `SO_BINDTODEVICE` (kernel lookup uses `idiag_if = 0`) — such flows fall back to the tunnel (safe, but the app stays "not excluded" for those sockets).
- Battery/CPU cost of userspace TCP for excluded apps under heavy use (e.g., excluding a streaming app). Mitigated by scope (only excluded flows) and gVisor's maturity (Tailscale/sing-box in production).
- Steering config changes (list edits) keep the existing semantics: applied on reconnect.
