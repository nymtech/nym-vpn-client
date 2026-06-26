# Bandwidth Cap — Engineering Specification

**Status:** Ready for implementation
**Owner:** TBD
**Last updated:** 2026-05-30
**Related docs:** `bandwidth-cap-proposal.md` (product context)

---

## 1. Background

In some Russian network environments, Android users report that sustained
high-throughput traffic over the Nym tunnel (e.g. video streaming) is cut
~5 minutes after the start of the session. The current working hypothesis
is **flow-shape fingerprinting**: the network classifier flags the
sustained high-bitrate WireGuard flow and drops the connection.

The Android team mitigated this in QA by enabling the OS-level
*"Limit downlink/uplink bandwidth"* developer setting at roughly 2 Mbps.
With that ceiling in place the cut-offs disappear, at the cost of
streaming quality.

We want to expose the same workaround as a first-class option in the
desktop app (`nym-vpn-app` + `nym-vpnd`), so users in hostile networks can
trade peak bandwidth for session stability without rooting their machine
or using OS-level QoS.

This document is the engineering spec for that work. It supersedes the
"Path B / token-bucket in Rust" sketch from `bandwidth-cap-proposal.md`,
which turned out to be infeasible without modifications to the
`nym-wg-go` Go library (see §3 for why).

## 2. Goals & non-goals

### Goals

1. User-controllable symmetric bandwidth cap on the active tunnel, with
   the cap surviving daemon restarts and reconnects.
2. Effective on **Linux** in this iteration; the wire protocol and UI
   must be designed so Windows/macOS can adopt the same surface later
   without a breaking change.
3. Live update: changing the slider while connected applies without a
   reconnect (or, if it requires a reconnect, the daemon reconnects
   itself transparently).
4. UI surfaced under *Settings → Anti-censorship*, alongside fronting
   mode, where the user mental model is "tools for hostile networks".

### Non-goals

1. Per-app or per-destination rate limiting.
2. Asymmetric up/down caps. (Symmetric is what the Android workaround
   uses and what the threat model needs.)
3. Adaptive / automatic rate adjustment based on observed conditions.
4. Implementing this in `nym-wg-go` itself in this iteration. We may
   revisit that as the path to a unified cross-platform implementation
   once the Linux version has shipped and we have evidence the feature
   is helping.

## 3. Why the in-process Rust token bucket was rejected

The proposal originally favored a Rust-side token bucket layered into
the WireGuard packet path (`Path B`). Investigation in `connected_tunnel.rs`
showed that both supported WG modes — `tun-tun` and `netstack` — call
into `nym-wg-go` (a Go FFI wrapper over wireguard-go) and from that
point on the data plane lives **entirely inside the Go runtime**:

- `run_using_tun_tun` (`connected_tunnel.rs:135`) starts
  `wireguard_go::Tunnel` instances and then enters a Rust event loop
  that handles only shutdown and (on Windows/iOS) default-route
  changes.
- `run_using_netstack` (`connected_tunnel.rs:274`) starts
  `netstack::Tunnel` and behaves the same way.

There is no Rust hot path between userspace and the tun device, so a
Rust-side limiter would have nothing to limit. Implementing the token
bucket inside `nym-wg-go` is possible but is a significant fork of a
vendored library and we don't want to block this user-facing fix on
that work.

## 4. Approach

**OS-level traffic control on the tun device**, configured by `nym-vpnd`
on Linux via `tc` (HTB qdisc + ingress policer), with the UI / control
plane generic so other platforms can drop in their own backend later
(Windows: `netsh interface ipv4 set subinterface` / WFP; macOS: `pfctl`
or `dnctl`).

The cap is **symmetric** by spec, but on Linux it's implemented as:

- **Egress** (`tx`, host → tunnel): root HTB qdisc on the tun device
  with a single rate-limited class.
- **Ingress** (`rx`, tunnel → host): an ingress qdisc + policer on the
  same device. (Ingress shaping on Linux is policing, not shaping — it
  drops over-rate packets. That's acceptable here; TCP will back off
  and the goal is to keep the flow shape below the classifier's
  threshold, not to be loss-free.)

Rate range: **500 kbps (0.5 Mbps) – 50 000 kbps (50 Mbps)**, default
**2 000 kbps (2 Mbps)** when enabled, default **off** overall.

## 5. Component overview

```
┌──────────────────────────────┐
│ React UI                     │
│  AntiCensorship screen       │
│   • Toggle                   │
│   • Slider 0.5–50 Mbps       │
└──────────────┬───────────────┘
               │ Tauri invoke
┌──────────────▼───────────────┐
│ Tauri command layer          │
│  set_bandwidth_cap            │
└──────────────┬───────────────┘
               │ gRPC (tonic)
┌──────────────▼───────────────┐
│ nym-vpnd command_interface   │
│  SetBandwidthCap handler     │
└──────────────┬───────────────┘
               │ mpsc VpnServiceCommand
┌──────────────▼───────────────┐
│ vpn_service.rs               │
│  handle_set_bandwidth_cap    │
└──────────────┬───────────────┘
               │
        ┌──────┴──────────┐
        │                 │
        ▼                 ▼
 config_manager      tc_shaper (new)
 set_bandwidth_cap   Linux: tc subprocess
 (persist to v10)    Other: no-op stub
```

## 6. Detailed changes

### 6.1 Wire protocol (`nym-vpn-proto`)

File: `nym-vpn-core/crates/nym-vpn-proto/proto/nym_vpn_service.proto`

Add (after `FrontingModeRequest` ~ line 1044):

```protobuf
// In-app bandwidth cap for the tunnel. Used to evade flow-shape
// fingerprinting on hostile networks (e.g. parts of Russia where
// sustained high-throughput VPN flows are killed after ~5 minutes
// of streaming). The rate is enforced symmetrically on tunnel
// ingress and egress.
message BandwidthCapRequest {
  bool enabled = 1;
  // Rate cap in kilobits per second. Range: 500 .. 50000. Ignored
  // when enabled = false. The daemon clamps out-of-range values
  // rather than rejecting the call, to keep the UI forgiving.
  uint32 rate_kbps = 2;
}
```

Add inside `service NymVpnService` (near `SetFrontingMode` ~ line 1164):

```protobuf
// Set or clear an in-app bandwidth cap on the tunnel. Applies live
// to an already-connected tunnel; persists across reconnects until
// cleared. Returns OK even when no tunnel is active — the setting
// is stored and applied at the next connect.
rpc SetBandwidthCap(BandwidthCapRequest) returns (google.protobuf.Empty) {}
```

Bindings (`ts-rs` output and Rust modules under
`nym-vpn-proto/src/`) regenerate from this proto via the existing
build pipeline; no hand-rolling required.

### 6.2 Config type (`nym-vpn-lib-types`)

File: `nym-vpn-core/crates/nym-vpn-lib-types/src/service.rs`

Add field to `VpnServiceConfig` (line 39):

```rust
/// In-app bandwidth cap (kilobits per second) applied symmetrically
/// on tunnel ingress and egress. `None` = no cap. When `Some`, the
/// value is in the inclusive range 500..=50_000 (daemon clamps).
pub bandwidth_cap_kbps: Option<u32>,
```

Update the `Default` impl (line 109) and the `Display` impl (line 61)
to include the new field. (Display: print `"bandwidth_cap_kbps: <value>"`
on its own line; leave it out when `None` to avoid log noise for the
common case.)

### 6.3 Persisted schema

File: `nym-vpn-core/crates/nym-vpn-lib/src/service/config/v10.rs`

Add field with `#[serde(default)]` so existing v10 configs on disk still
parse:

```rust
#[serde(default)]
pub bandwidth_cap_kbps: Option<u32>,
```

Update the two converter impls (`TryFrom<VpnServiceConfig>` for the
in-memory type, and `TryFrom<&nym_vpn_lib_types::VpnServiceConfig>` for
v10 in `mod.rs:219`) to round-trip the field.

Update all v10 test fixtures in
`nym-vpn-core/crates/nym-vpn-lib/src/service/config/tests.rs` — most are
JSON snapshots that include every field; add `"bandwidth_cap_kbps": null`
to them, or set them to specific values where the test is exercising the
field.

**No new version (v11) is required**: `#[serde(default)]` makes the
addition backward-compatible at the persistence layer. Bump to v11 only
if/when we add a field that needs migration logic.

### 6.4 Service command + handler

File: `nym-vpn-core/crates/nym-vpn-lib/src/service/vpn_service.rs`

Add variant to `VpnServiceCommand` (around line 101, near `SetFrontingMode`):

```rust
SetBandwidthCap(oneshot::Sender<()>, Option<u32>),
```

`None` means "disable cap"; `Some(kbps)` means "enable at kbps".

Dispatch in the command loop (around line 980):

```rust
VpnServiceCommand::SetBandwidthCap(tx, cap) => {
    self.handle_set_bandwidth_cap(cap).await;
    let _ = tx.send(());
}
```

Handler (alongside `handle_set_fronting_mode` ~ line 1327):

```rust
async fn handle_set_bandwidth_cap(&mut self, cap: Option<u32>) {
    let clamped = cap.map(|k| k.clamp(500, 50_000));
    self.config_manager.set_bandwidth_cap(clamped).await;

    // Apply live if a tunnel is currently up. The shaper module
    // owns idempotency: applying the same cap twice is a no-op.
    if let Some(tun_name) = self.active_tunnel_iface() {
        if let Err(e) = crate::shaper::apply(&tun_name, clamped).await {
            tracing::warn!("Failed to apply bandwidth cap: {e}");
        }
    }
}
```

`active_tunnel_iface()` is a new helper that returns the tun interface
name of the active wireguard tunnel (or `None` if disconnected). For
the netstack mode the host doesn't have a real tun device, so this
method returns `None` and the cap is silently ignored — document that
limitation in the UI (§6.8).

### 6.5 Config manager

File: `nym-vpn-core/crates/nym-vpn-lib/src/service/config/config_manager.rs`

Add (mirroring `set_fronting_mode` at line 168):

```rust
pub async fn set_bandwidth_cap(&mut self, cap: Option<u32>) {
    if self.config.bandwidth_cap_kbps != cap {
        self.config.bandwidth_cap_kbps = cap;
        if let Err(e) = self.persist().await {
            tracing::error!("Failed to persist bandwidth cap: {e}");
        }
    }
}
```

### 6.6 Shaper module

New file: `nym-vpn-core/crates/nym-vpn-lib/src/shaper/mod.rs`

```rust
// Cross-platform interface; only Linux has a real backend in this iteration.
pub async fn apply(iface: &str, cap_kbps: Option<u32>) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    return linux::apply(iface, cap_kbps).await;
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (iface, cap_kbps);
        tracing::debug!("bandwidth cap requested but not supported on this platform");
        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod linux;
```

New file: `nym-vpn-core/crates/nym-vpn-lib/src/shaper/linux.rs`

```rust
use tokio::process::Command;

// HTB egress + ingress policer on the tun device.
// We always tear down and re-add — `tc qdisc del` is idempotent (the
// "RTNETLINK: No such file or directory" failure is ignored).
pub async fn apply(iface: &str, cap_kbps: Option<u32>) -> std::io::Result<()> {
    teardown(iface).await.ok();
    let Some(kbps) = cap_kbps else { return Ok(()); };

    // Egress: root HTB, one class at `kbps`.
    run("tc", &["qdisc", "add", "dev", iface, "root", "handle", "1:", "htb", "default", "10"]).await?;
    run("tc", &["class", "add", "dev", iface, "parent", "1:", "classid", "1:10",
                "htb", "rate", &format!("{kbps}kbit"), "ceil", &format!("{kbps}kbit")]).await?;

    // Ingress: policer at `kbps`, drop overflow.
    run("tc", &["qdisc", "add", "dev", iface, "ingress"]).await?;
    run("tc", &["filter", "add", "dev", iface, "parent", "ffff:", "protocol", "all", "u32",
                "match", "u32", "0", "0",
                "action", "police", "rate", &format!("{kbps}kbit"),
                "burst", &format!("{burst}k", burst = (kbps / 8).max(16)),
                "drop"]).await?;
    Ok(())
}

async fn teardown(iface: &str) -> std::io::Result<()> {
    let _ = run("tc", &["qdisc", "del", "dev", iface, "root"]).await;
    let _ = run("tc", &["qdisc", "del", "dev", iface, "ingress"]).await;
    Ok(())
}

async fn run(prog: &str, args: &[&str]) -> std::io::Result<()> {
    let out = Command::new(prog).args(args).output().await?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        // tc returns nonzero when removing a qdisc that doesn't exist;
        // callers above use `.ok()` to swallow that, so just propagate.
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{prog} {args:?} failed: {stderr}"),
        ));
    }
    Ok(())
}
```

The daemon already runs as root on Linux (it manages `nym-firewall` /
`nft`), so no additional privileges are needed.

### 6.7 Apply on connect

In the tunnel state machine, after the WG tunnel reports
`TunnelState::Connected` and the tun interface name is known, the
service must call `shaper::apply(iface, config.bandwidth_cap_kbps)`.
The natural hook is wherever fronting mode / firewall rules are applied
post-connect — follow the same pattern. On disconnect the tun device is
torn down so the qdiscs vanish automatically; no explicit cleanup
needed.

### 6.8 RPC client + tonic handler

Wrapper in `nym-vpn-core/crates/nym-vpn-proto/src/rpc_client.rs`,
following the `set_fronting_mode` pattern at line 253–256:

```rust
pub async fn set_bandwidth_cap(&mut self, enabled: bool, rate_kbps: u32) -> Result<()> {
    let req = BandwidthCapRequest { enabled, rate_kbps };
    self.client.set_bandwidth_cap(req).await?;
    Ok(())
}
```

Tonic handler in `nym-vpn-core/crates/nym-vpnd/src/command_interface.rs`,
following the `SetFrontingMode` handler at line 192–210:

```rust
async fn set_bandwidth_cap(
    &self,
    request: Request<BandwidthCapRequest>,
) -> Result<Response<()>, Status> {
    let req = request.into_inner();
    let cap = req.enabled.then_some(req.rate_kbps);
    let (tx, rx) = oneshot::channel();
    self.command_tx
        .send(VpnServiceCommand::SetBandwidthCap(tx, cap))
        .map_err(|_| Status::internal("service unavailable"))?;
    let _ = rx.await;
    Ok(Response::new(()))
}
```

### 6.9 Tauri layer

File: `nym-vpn-app/src-tauri/src/vpnd/client.rs`

Add a wrapper that calls the new gRPC method.

File: `nym-vpn-app/src-tauri/src/commands/tunnel.rs`

```rust
#[tauri::command]
pub async fn set_bandwidth_cap(
    state: State<'_, AppState>,
    enabled: bool,
    rate_kbps: u32,
) -> Result<(), String> { /* … */ }
```

Register the command in `lib.rs` next to the other `tunnel` commands.

### 6.10 Frontend

File: `nym-vpn-app/src/screens/settings/anti-censorship/AntiCensorship.tsx`

Add a section below the fronting-mode controls:

```
Bandwidth cap                                          [toggle]
Limits the tunnel's throughput in both directions. Use this if
your network drops the VPN after sustained high-speed traffic.

[──────●─────────────] 2 Mbps
0.5 Mbps           50 Mbps
```

- Slider step: 0.5 Mbps (i.e. 500 kbps).
- Disabled state when toggle is off; the slider keeps its last
  selected value so re-enabling restores it.
- On change (toggle or slider), debounce ~300 ms and call
  `set_bandwidth_cap`.

Add `bandwidthCap: { enabled: boolean; rateKbps: number }` to the
Zustand main slice (`src/store/slices/createMainSlice.ts`) with a
`setBandwidthCap` action that updates state optimistically and calls
the Tauri command; if the Tauri call fails, roll back and surface a
toast.

i18n: add `settings.antiCensorship.bandwidthCap.*` strings (title,
description, enabled-label, slider-units, error toast). Keys follow
the existing `settings.antiCensorship.frontingMode.*` shape.

**Platform UX notes:**

- Windows/macOS desktop: gray the section out with a
  "Coming soon on this platform" tooltip until a backend exists.
- Mixnet-only / 5-hop mode: gray out with "Available in 2-hop mode
  only" — the cap operates on the WireGuard tun device.

## 7. Testing

### Unit

- `config_manager::set_bandwidth_cap` — round-trips and persists
  correctly; clamps to range; no-op on equal value.
- `shaper::linux::apply` — given a fake tun, invokes the expected
  `tc` argv sequence (the test stubs `Command` via a trait or just
  asserts on a recorded argv vec). Validate that disable-after-enable
  produces the right teardown ordering.
- v10 schema: serializing then deserializing a config with the field
  set produces the same value; deserializing a v10 JSON *without* the
  field yields `bandwidth_cap_kbps: None` (this is the migration
  safety test).

### Integration

- Bring up the daemon in a Linux integration test with a stub tun
  device (e.g. `dummy0` configured up + `ip tuntap add`), call
  `SetBandwidthCap(enabled=true, rate_kbps=2000)`, and assert via
  `tc -s qdisc show dev <iface>` that the htb root and ingress
  qdiscs exist with the expected rates.

### Manual

1. Connect on Linux, enable cap at 2 Mbps, run
   `iperf3 -c <speedtest>` and confirm throughput is ~2 Mbps both
   directions.
2. Move slider to 10 Mbps while connected; throughput updates
   without reconnect.
3. Disable the toggle; throughput returns to line rate.
4. Restart the daemon while cap is enabled at 5 Mbps; reconnect;
   confirm cap is reapplied to the new tun device.
5. Russia-emulating soak test: cap at 2 Mbps, run sustained
   download for 30 minutes, verify the tunnel is not cut.

## 8. Out-of-scope follow-ups

- Windows backend: WFP filter or `netsh int ipv4 set subinterface`
  on the wintun adapter.
- macOS backend: `dnctl` pipes + `pfctl` anchor, written from the
  daemon (already root via launchd).
- A unified implementation inside `nym-wg-go` (would let us delete
  the platform-specific shaper modules and use a single token
  bucket regardless of OS). Reassess once we have field data on
  whether the cap actually keeps Russian streaming sessions alive.
- Telemetry: count of (capped, uncapped) connection sessions and
  duration distribution, to validate the hypothesis empirically.

## 9. Risks

- **Netstack mode**: there is no host tun device to attach `tc` to,
  so the cap is a silent no-op. UI must surface this; spec says we
  gray the control out in netstack mode.
- **Ingress policing drops packets**: TCP recovers but UDP-heavy
  workloads (QUIC streaming) will see degradation beyond just slow.
  This is acceptable — the threat model is flow-shape evasion, not
  smooth playback — but document it in the user-facing tooltip.
- **tc unavailable**: minimal Linux distros (Alpine in containers,
  some embedded) may not ship `iproute2`. Daemon must surface a
  clear error to the UI rather than silently failing. Phase-2
  enhancement: probe `tc --version` once at startup and disable the
  feature in the UI capabilities response.
