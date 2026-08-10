# Android: split tunneling under always-on VPN lockdown

**Date:** 2026-08-07 (design); updated 2026-08-10 with as-built findings
**Status:** Implemented on branch `feat/android-split-tunnel-always-on`; validated end-to-end on a physical device (see "Implementation notes & on-device validation" below). Two runtime behaviours differ from the original design and are documented inline.
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
| Excluded apps, lockdown **on** (API 29+) | **Flow steering:** no `addDisallowedApplication`; a steering layer owns the TUN fd and forwards excluded apps' flows directly over protected sockets |

On API 24–28 lockdown cannot be detected (the detection below is API 29+); the split-tunneling screen shows a static informational note that lockdown will block excluded apps (see UI section).

> **As-built correction — lockdown detection (2026-08-10).** The design assumed `VpnService.isLockdownEnabled()` alone is sufficient. On device it is **not reliable**: it returned `false` on an already-running `VpnService` even when the user had lockdown enabled (it appears to reflect the lockdown state captured when the service was started under the always-on path, so an app-initiated connect on a running process misses it). Relying on it alone meant steering never engaged and excluded apps stayed blocked — the exact bug this feature fixes.
>
> The shipped gate is `frameworkLockdown || persistedLockdown`, where `frameworkLockdown = isLockdownEnabled()` and `persistedLockdown` reads `Settings.Secure`. **Android-16 fact:** an app *can* read `always_on_vpn_lockdown` (returns `1` when enabled) but *cannot* read `always_on_vpn_app` (returns `null` — system-restricted). So the persisted check is `always_on_vpn_lockdown == 1 && (always_on_vpn_app == null || always_on_vpn_app == ourPackage)` — a `null` app-name is treated as "unknown → trust the flag"; only a readable *mismatch* vetoes. A false positive is harmless (excluded flows go direct over protected sockets whether or not lockdown is actually enforced). Implemented as the pure, unit-tested `AppBypassResolver.isLockdownActive(...)`. The same `always_on_vpn_app`-read bug affects the app-module display helper `GeneralExtensions.isVpnLockdownEnabled` (why the Task-9 lockdown card never rendered under lockdown) — it needs the same null-handling.

### The steering layer (Go, in `wireguard/libwg`)

A new Go component, following the pattern of the existing `dns_filter_proxy.rs` (socketpair-as-fake-TUN) and reusing `libwg`'s existing gVisor netstack and socket↔netstack forwarders (`forwarders/tcp.go`, `forwarders/udp.go`).

**Placement:** Rust creates an `AF_UNIX SOCK_DGRAM` socketpair. The steering layer gets the real TUN fd plus one end of the pair; the existing downstream consumer — wireguard-go (optionally behind the DNS filter proxy, which chains with the same shim pattern) or the Rust `MixnetProcessor` — gets the other end and is otherwise unchanged. Go is chosen over Rust because gVisor and the forwarders already live in `libwg`, and `libwg` is linked into the Android build in both modes.

**Packet path (upstream, from apps):**
1. Read raw IP packet from the real TUN fd.
2. Look up the flow (5-tuple) in a flow table.
3. **New flow:** classify — call the attribution callback (below) → UID → in excluded-UID set?
   - *Excluded:* mark flow BYPASS; inject packet into a dedicated gVisor netstack instance. TCP is terminated there and bridged to a real socket dialed on the underlying network; UDP flows get a NAT-style relay socket. Every outbound socket fd is passed through the existing bypass callback (`AndroidTunProvider.bypass(fd)` → `VpnService.protect()`) before connect. **These `protect()` calls MUST be serialized (see the concurrency note under Error handling) — this is load-bearing, not optional.**
   - *Tunneled, unattributable (`INVALID_UID`), or lookup error:* mark flow TUNNEL (fail-closed) and write the packet to the socketpair.
4. **Known flow:** route per the cached mark. No per-packet attribution calls.

**Packet path (downstream, to apps):** merge packets from the socketpair (tunnel) and the bypass netstack, write to the real TUN fd.

**Efficiency:** tunneled traffic pays one extra fd copy — the same cost the shipping DNS-filter (ad-block) proxy already imposes on the WG path. Userspace TCP termination applies only to excluded apps' flows.

### Attribution and configuration plumbing

The excluded-app list currently lives only in Kotlin and is applied only via the builder. New plumbing:

- **Excluded UIDs → core:** Kotlin resolves package names to UIDs (`PackageManager.getApplicationInfo().uid`) at connect/reconnect time and sends the UID set plus a `steeringEnabled` flag to Rust via a new `NymVpnServiceCommandSender` setter, alongside the existing ones dispatched in `VpnCoreController.applyConfigDiffToSender()`. Rust stores it next to the other per-connection options and threads it into `TunnelSettings` (`nym-vpn-lib/src/tunnel_provider/mod.rs`) and from there into the steering layer's config.
- **Connection-owner callback:** extend the `AndroidTunProvider` uniffi trait (`nym-vpn-lib-uniffi/src/tunnel_provider/android.rs`) with `get_connection_owner_uid(protocol: i32, source: String, destination: String) -> i32` (addresses are passed as strings in Go `netip.AddrPort` form — `"ip:port"` / `"[v6]:port"` — not `SocketAddr`, to keep the FFI simple), implemented in Kotlin via `ConnectivityManager.getConnectionOwnerUid()`. Called once per new flow from Go via the existing Rust↔Go boundary; result cached in the flow table. Clash Meta uses the identical call pattern without rate-limit issues. Fail-closed at every layer (bad address string / `SecurityException` / API < 29 → `-1`).

> **As-built note — interface name via `TUNGETIFF` (2026-08-10).** `create_tun_device` (both Android modes) previously derived the tunnel interface name from the device fd with a `TUNGETIFF` ioctl. That ioctl fails on the steering socketpair fd, and the name is load-bearing — it's the `SO_BINDTODEVICE` target for the wg-metadata socket. Fix: resolve the interface name from the **real** TUN fd *before* handing it to the steering engine, then return `(AsyncDevice, name)` from `create_tun_device`. (Guard this resolution `#[cfg(target_os = "android")]`; on iOS the early-return path can otherwise close the system-owned utun fd.)

### DNS for excluded apps

Excluded apps' DNS queries arrive in the TUN addressed to the tunnel's DNS server. If forwarded through the tunnel, "direct" apps would still resolve through Nym (functional and privacy mismatch, and broken if the tunnel is down while lockdown holds). Instead: Kotlin obtains the underlying (non-VPN) network's resolvers via `ConnectivityManager` link properties and passes them in the steering config; the steering layer rewrites excluded flows' DNS destination to those resolvers and relays over protected sockets. If no underlying resolvers are known, excluded apps' DNS falls back to the tunnel path (name resolution keeps working; only the resolver path is non-direct in that degraded case).

### IPv6

The TUN already claims IPv6 routes (`compute_tunnel_networks`). Excluded v6 flows are forwarded via protected v6 sockets when the underlying network has IPv6; otherwise the dial fails and the app's Happy Eyeballs falls back to v4. No NAT64.

### Error handling

- Attribution `SecurityException` / `INVALID_UID` / callback failure → flow goes through the tunnel. Never bypass on uncertainty.
- Bypass dial failure → TCP RST / ICMP port-unreachable synthesized back to the app (netstack default), so apps fail fast instead of hanging.
- Steering-layer fatal error → tear down the tunnel through the normal error path (fail-closed; lockdown then blocks everything, which is the user's chosen posture).
- Flow table: LRU + idle timeouts (TCP by state, UDP ~60 s) to bound memory.

> **As-built addition — serialize `protect()` (2026-08-10).** The bypass netstack dials one socket per excluded-app flow and protects it from a concurrent per-flow goroutine. Running `VpnService.protect()` → `protectFromVpn` concurrently (it opens/closes its own netd control fd via a libbase `unique_fd`) races with the Go runtime's fd churn on other goroutines, and bionic **fdsan aborts the process** ("double-close" / "close of fd owned by `unique_fd`"). On device this reliably crashed the steering-active connect, so the tunnel never completed. Fix: serialize all `protect()` calls — the shipped code wraps `VpnService.bypass()` in `synchronized(protectLock)`, keeping at most one `protectFromVpn` alive at a time (matching how the never-crashing 2-hop entry sockets are protected serially at startup). Verified: 10+ steering-active connect retries with zero `protectFromVpn` aborts where before it crashed on the first. (Unrelated: the device's Mali GPU driver, `libGLES_mali.so`, has its own fdsan double-close during UI rendering — not ours.)

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

## Implementation notes & on-device validation (2026-08-10)

**End-to-end validation** (Volla Phone X23, Android 16 / API 36, WireGuard 2-hop, lockdown enabled via the system settings toggle, Firefox/Fennec excluded, uid 10072):

- Steering activated on a normal app-initiated connect: `app_bypass: Some(excluded_uids: [10072], underlying_dns: [...])`, engine started, `addDisallowedApplication` skipped.
- OS-level confirmation: the VPN `NetworkAgent` covered **`Uids: <{0-99999}>`** (no exclusion gap) — i.e. steering in-tunnel, not classic OS exclusion. (The classic path shows a single-uid gap.)
- No `protectFromVpn` fdsan crash; process stable.
- **Behavioural proof, simultaneously under lockdown:** the excluded browser egressed from the device's real cellular IP `178.197.161.237` (direct), while tunneled traffic (adb shell, uid 2000) egressed from the Nym exit `87.106.222.6`. The excluded app loaded pages *at all* under lockdown — impossible with plain `addDisallowedApplication`.

**Build / packaging gotchas** (surface with the local `Android.mk` build; do a full `make -f Android.mk` + `assembleGeneralDebug`, not `-PbuildDeps=false`):
- `libwg.so` must be present in `jniLibs/<abi>/` for all three ABIs. `build-wireguard-go.sh` output lands in `build/lib/<rust-triple>/`; the full `buildDeps` step places it — a `-PbuildDeps=false` shortcut ships an APK missing `libwg.so`, which crashes at load (`libnym_vpn_lib.so` has a DT_NEEDED on it, and `extractNativeLibs=false` makes a missing lib fatal).
- Regenerate uniffi Kotlin bindings against the merged library (`uniffi-bindgen generate --library …/libnym_vpn_lib.so`), so every namespace loads from the single `libnym_vpn_lib.so`. A stale per-crate binding (e.g. `nym_bridges_types` loading a non-existent `libnym_bridges_types.so`) crashes core init.

**Environmental note:** connect completion is independent of this feature — on a flaky cellular-roaming link the tunnel retries regardless of lockdown/steering; on a healthy link it connects in ~1 s. Earlier "stuck connecting" observations were the network, not steering.

## Risks / open questions

- `getConnectionOwnerUid` misses sockets bound with `SO_BINDTODEVICE` (kernel lookup uses `idiag_if = 0`) — such flows fall back to the tunnel (safe, but the app stays "not excluded" for those sockets).
- Battery/CPU cost of userspace TCP for excluded apps under heavy use (e.g., excluding a streaming app). Mitigated by scope (only excluded flows) and gVisor's maturity (Tailscale/sing-box in production).
- Steering config changes (list edits) keep the existing semantics: applied on reconnect.
- **Known follow-up — gVisor UDP packet-buffer leak.** The gVisor version pinned in `libwg` exposes no `Release()`/`DecRef` on `udp.ForwarderRequest`, so the bypass netstack leaks ~one `PacketBuffer` per bypassed UDP flow. DNS is the hot UDP path, so this accumulates over a long lockdown session. Needs a gVisor bump or a manual buffer-drop. Not a merge blocker, but a real leak to schedule.
