# Bandwidth-cap option for Wi-Fi networks hostile to streaming VPN traffic

**For:** Product + engineering review
**Status:** Proposal, awaiting decision
**Engineering effort:** ~2 dev-days (Linux first), ~+1 day each for Windows / iOS / Android
**Backend infrastructure needed:** none

---

## Summary

Some Russian (and likely Iranian, Belarusian, Turkmen, Chinese-edge-of-network) ISPs run **flow-shape fingerprinting** that kills VPN connections producing sustained high-throughput traffic — typically after ~5 minutes of streaming through the tunnel. Users have reproducibly worked around this by enabling Android's developer-mode bandwidth limiter and capping the phone at ~1–2 Mbps; the same VPN session that gets killed at uncapped throughput sustains indefinitely at the capped rate.

We should expose this knob in nym-vpn directly, so users in these environments don't have to mess with developer settings or know about the workaround. **Linux first, then Windows, then mobile.**

Decision asked: yes/no on shipping this, plus a one-line yes/no on which shaping path to take (kernel-level `tc` vs Rust app-level — recommendation included below).

---

## Why the existing mitigations don't help

The user's network is on a regime that is **actively profiling traffic shape**, not just blocking known VPN endpoints. So the things we already ship don't apply:

| Existing mitigation | Why it doesn't help here |
|---|---|
| QUIC bridges (Anti-censorship → QUIC) | Masquerades the *transport* as HTTPS, but a sustained 30 Mbps HTTPS flow still looks anomalous and gets killed |
| Domain fronting | Hides *what* the connection is going to, not *how much* it's pumping |
| Mixnet (Anonymous mode) | Lower throughput by design, but slower than streaming needs — and users want streaming |

The fingerprinting isn't "is this VPN traffic?" — it's "is this flow shaped like residential browsing?" Capping at 1–2 Mbps makes a wg-over-QUIC flow look approximately like normal video conferencing or HTTPS browsing, which the regime tolerates.

## The empirical workaround that works

From multiple Russian users:

> Settings → System → Developer options → Networking → **Mobile data always active** + **Mobile data bandwidth limit** = 2048 kbps (or thereabouts). Stream content through the VPN — connection holds. Remove the limit — connection drops within ~5 minutes.

So the question is just: replicate that throttle inside the app instead of asking users to enable developer mode.

## Proposed feature

**Settings → Anti-censorship → "Cap tunnel bandwidth"**

```
┌────────────────────────────────────────────────────────────┐
│ Cap tunnel bandwidth                          [ off | on ] │
│                                                            │
│ Limits how fast data flows through the VPN. Some networks  │
│ (including parts of Russia, Iran, and other restricted     │
│ regions) drop VPN connections that move data too quickly.  │
│ Capping the rate keeps the connection alive at the cost of │
│ slower downloads.                                          │
│                                                            │
│ Cap (Mbps):  ●━━━━━━━○──────────  2 Mbps                   │
│              0.5            10                             │
└────────────────────────────────────────────────────────────┘
```

- **Default: off.** Most users don't need it; on it kneecaps their throughput.
- **When on, default rate: 2 Mbps.** Matches the workaround users found empirically. Tunable 0.5–10 Mbps via slider.
- Lives next to the existing QUIC / Domain Fronting toggles — same conceptual bucket ("things you turn on when your network is hostile").
- Apply takes effect immediately if connected, or on next connect if disconnected. No reconnect needed.

## Where to implement the shaping

Two paths; pick one.

### Path A — Linux kernel `tc` (Traffic Control)

Install a token-bucket queueing discipline on the daemon's tunnel interface using the kernel's built-in `tc` subsystem.

```sh
# applied automatically by the daemon when the user enables the cap:
tc qdisc add dev nymtun0 root tbf rate 2mbit burst 32kbit latency 50ms
# for ingress (download), needs an ifb shim:
ip link add ifb-nym type ifb && ip link set ifb-nym up
tc qdisc add dev nymtun0 handle ffff: ingress
tc filter add dev nymtun0 parent ffff: u32 match u32 0 0 action mirred egress redirect dev ifb-nym
tc qdisc add dev ifb-nym root tbf rate 2mbit burst 32kbit latency 50ms
```

| Pros | Cons |
|---|---|
| Best precision — kernel scheduling, sub-millisecond | Linux-only. iOS/Android/Windows need separate implementations |
| Zero CPU on the daemon (kernel does the work) | Needs `CAP_NET_ADMIN` (already have it — daemon runs as root) |
| Survives a busy event loop in the Rust daemon | Brittle: depends on tunnel-interface name detection; partial cleanup on crash is possible |
| | Three different paths for one feature across platforms |

### Path B — Rust token-bucket on the daemon's WireGuard transport (**recommended**)

Wrap the read/write side of the tunnel's WireGuard packet handler in `nym-vpn-lib` with a token-bucket rate limiter (~30 lines of Rust). Configurable at the `VpnServiceConfig` level via a new `SetTunnelBandwidthCap(rate_kbps)` gRPC RPC.

| Pros | Cons |
|---|---|
| Cross-platform — one implementation works on Linux, Windows, Android (UniFFI), iOS (XPC) | Less precise than kernel-level (millisecond-ish, depending on async loop scheduling). Fine for adversarial traffic-shape fingerprinting that operates over multi-second windows |
| No privilege concerns, no interface-name detection | Uses a bit of daemon CPU (negligible at 2 Mbps) |
| Composable with future features (e.g. per-flow caps for split tunneling) | |
| Same code path is what most software VPNs use | |

The fingerprinting threshold operates over multi-second windows ("flow looks like streaming"), not microsecond precision. Path B is good enough and ships in 1/3 the time.

**Recommendation: Path B.** Use kernel `tc` only if measured user latency is materially worse than `tc` would give — and that's vanishingly unlikely at the rates we're capping at.

## Implementation outline (Path B, Linux first)

1. **Daemon (`nym-vpn-core/crates/nym-vpn-lib`)**
   - Add `bandwidth_cap_kbps: Option<u32>` field to `VpnServiceConfig`. `None` = uncapped (default).
   - In the WireGuard transport read/write path, wrap I/O with a `governor::RateLimiter` (the crate is already in the workspace, or `async-throttle`).
   - Apply on tunnel up; re-apply on config change without forcing a reconnect.
2. **Proto (`nym-vpn-core/crates/nym-vpn-proto/proto/nym_vpn_service.proto`)**
   - New RPC: `rpc SetTunnelBandwidthCap(BandwidthCapRequest) returns (Empty);` where `BandwidthCapRequest { uint32 rate_kbps; bool enabled; }`.
3. **Tauri Rust (`nym-vpn-app/src-tauri/`)**
   - `vpnd/client.rs` — wrap the new RPC.
   - `commands/tunnel.rs` — new `set_bandwidth_cap` Tauri command.
   - `vpnd/config/vpnd_config.rs` — surface the field through to the frontend via the existing config-state event.
4. **Tauri frontend (`nym-vpn-app/src/`)**
   - `screens/settings/anti-censorship/AntiCensorship.tsx` — add the toggle + slider.
   - `store/slices/createMainSlice.ts` — new `bandwidthCap: { enabled: bool, rateKbps: number }` field.
   - i18n: ~6 new strings (`title`, `description`, `cap-label`, `learn-more`, on/off labels).
5. **No daemon proto change for Windows / Android / iOS** — same RPC works for all once it exists. Surfacing the toggle in those frontends is a separate ticket per platform.

## What we explicitly don't do

- **No automatic detection** of "you're on a Russian network, suggest enabling this." Detection of where the user is and tailoring suggestions accordingly is its own scope and has privacy implications. The setting is user-discoverable in Anti-censorship; manual.
- **No per-app / split-tunneling rate caps.** One global cap. Maybe a v2 if asked.
- **No upload/download asymmetry in the UI.** Single Mbps value applied symmetrically. Adding two sliders for asymmetric caps doubles the UI and isn't what the workaround is doing on Android either.
- **No bandwidth-usage UI.** This is a cap, not a meter — we already have a meter (network stats).

## Risks and edge cases

- **Existing connections at the moment of toggle.** Applying the cap should not drop the tunnel; the rate limiter just starts throttling. If we get this wrong, users have to reconnect — annoying but recoverable. Worth specific test coverage.
- **Below ~500 kbps the user experience degrades sharply** (web pages stall, even messaging gets sluggish). The slider's lower bound at 0.5 Mbps reflects this.
- **Mixnet (Anonymous) mode already runs at <5 Mbps in practice** — the cap is a no-op there. We could grey out the toggle when in Anonymous mode for clarity, or just let it silently no-op. I'd let it no-op; less UI logic.
- **iOS network extension memory limits** are stricter than other platforms — if we move buffered bytes around in user-space, this might bite. Worth a quick test before promising iOS parity.

## Decision

1. **Should we ship this?** Yes / no.
2. **Path A (kernel `tc`) or Path B (Rust app-level)?** Recommendation: B.
3. **Default rate when enabled — 2 Mbps?** (Recommendation: yes — matches the empirical workaround. Slider 0.5–10 Mbps.)

---

*This proposal is a response to a confirmed user report from Russian Android users describing connection drops within ~5 minutes of streaming through nym-vpn, mitigated by enabling Android developer-mode network bandwidth throttling at ~2 Mbps. The "Anti-censorship" settings group in the app is the natural home for the toggle.*
