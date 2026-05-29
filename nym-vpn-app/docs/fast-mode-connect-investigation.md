# Fast-mode connect failure on Linux — investigation

**Symptom:** in the Tauri Linux client, switching to **Fast** (WireGuard) mode and pressing **Connect** fails to bring the tunnel up. The same nymd, same network env, same account: the **Android** client connects immediately. Kill switch is off.

**Most likely cause (cross-checked against the Android daemon's logged `Using config: VpnServiceConfig {...}` dump):** the Tauri client never calls **`SetNetstack`**, leaving the daemon at its default `netstack: false`. The Android daemon runs with `netstack: true` for the same Fast (WG) connect. With `netstack: false` the daemon picks a different second-hop implementation that needs network capabilities Tauri's process may not have, or that handshakes differently and silently fails on this network.

Concrete evidence — the Android daemon dumps its full `VpnServiceConfig` in the logcat. Two successful Fast connects on `2026-05-28` 23:16 and 23:18 both ran with:

```
VpnServiceConfig {
    entry_point: Country { CH }, exit_point: Random,
    enable_two_hop: true,             ← Fast / WG
    enable_bridges: false,            ← same as Tauri default
    fronting_mode: OnRetry,           ← same as Tauri default (lib-types default)
    netstack: true,                   ← THE DIFFERENCE
    ...
    wireguard_tunnel_options: WireguardTunnelOptions {
        multihop_mode: Netstack,      ← driven by netstack=true
        enable_bridges: false,
    }
}
```

Earlier hypothesis (default-mismatch on QUIC) was wrong: both clients default `enable_bridges = false` and `fronting_mode = OnRetry`, so the QUIC bridge fallback fires on retry for both. The Android log proves this: the 23:16 attempt failed direct first (`Tunnel connection is failing (retry: 1)`) then succeeded `via bridge 94.232.246.49:4443` because `OnRetry` upgraded it. **The 23:18 attempt connected direct on the first retry without bridge** — so QUIC isn't even strictly required for Fast.

**Quick user-side checks to verify, in order of how much they tell us:**

1. Settings → Anti-censorship → toggle **QUIC** on → try Fast. If it works, `enable_bridges: true` is masking the underlying netstack issue (because the bridged path uses a different transport layer entirely).
2. Provide the full Tauri daemon log (see §8) so we see _its_ `Using config: VpnServiceConfig {...}` dump. That will confirm `netstack: false` and identify the specific failure mode the daemon hit.

There's also a secondary architectural gap on Tauri (see §3) where the connect path doesn't re-assert the daemon's full state before `ConnectTunnel`. That doesn't apply to `netstack` (which has no Tauri command), but it's relevant for entry/exit/algorithm coherence.

This document is for whoever owns `nym-vpn-app/src-tauri/src/commands/tunnel.rs` and `nym-vpn-app/src-tauri/src/vpnd/client.rs`. The "Fix" section lists concrete, minimal changes.

---

## 1. What the daemon needs before `ConnectTunnel`

`nym-vpn-core/crates/nym-vpn-proto/proto/nym_vpn_service.proto` exposes setters on `NymVpnService` (line numbers from current `develop`):

- `SetEntryPoint(EntryNode)` — line 1136
- `SetExitPoint(ExitNode)` — line 1137
- `SetEnableTwoHop(BoolValue)` — line 1139 (`true` = WireGuard / Fast, `false` = Mixnet)
- `SetEnableBridges(BoolValue)` — line 1140 (QUIC fronting)
- `SetGatewaySelectionAlgorithm(GatewaySelectionAlgorithm)` — line 1152
- `SetFrontingMode(FrontingModeRequest)` — line 1154
- `SetResidentialExit`, `SetNetstack`, `SetCustomDns`, …
- `ConnectTunnel(Empty)` — line 1172

The daemon holds the full config in memory; setters mutate that state. `ConnectTunnel` reads the _current_ state and acts on it. There is no atomic "connect-with-this-config" RPC. **It's the client's job to make the daemon's state match the user's intent before calling `ConnectTunnel`.**

---

## 2. How Android does it (verified in `VpnCoreController.kt`)

`connect()` → `connectLocked()` (line 163) runs three steps in order:

1. `ensureCoreInitialized(...)` (line 175) — first-call only; pushes a baseline `VpnConfig` to the Rust core, including `gatewayIndependence = GatewayIndependence(differentNodeFamily=true, differentAsn=true)` and `frontingMode = ON_RETRY` (line 359, 349). On Android this goes through UniFFI, not gRPC.
2. **`applyCanonicalConfigToRustIfReady(force=false, canonical=null)`** (line 190) — diffs the canonical app config against `lastAppliedConfig` and pushes every changed setter:
   - `setEnableTwoHop(cfg.mode.isTwoHop())`
   - `setGatewaySelectionAlgorithm(cfg.algorithm)`
   - `setEnableBridges(cfg.enableBridges)`
   - `setEnableCustomDns(cfg.customDnsEnabled)`
   - `setCustomDns(cfg.customDns)` (if enabled)
   - `setEntryPoint(cfg.entryPoint)`
   - `setExitPoint(cfg.exitPoint)`
   - `setEnableAdBlocking(cfg.adBlockingEnabled)`

   On the **first** connect after a fresh start `lastAppliedConfig` is `null`, so every comparison fails and **everything fires.** That's the safety net: the daemon's state is fully re-asserted from the UI's canonical config before each connect.

3. `connectTunnel()` (line 198).

If any setter fails, the whole connect bails with `ApplyConfigBeforeConnectFailed` — no half-applied state.

---

## 3. How Tauri does it today (verified in `src-tauri/`)

`commands/tunnel.rs:38-93` — `connect`:

```rust
pub async fn connect(...) -> Result<TunnelState, BackendError> {
    // 1. assert disconnected, transition local UI state to Connecting
    // 2. log current vpnd_config fields
    // 3. emit "connecting" event
    // 4. vpnd.vpn_connect().await   ← this is the ONLY daemon call
}
```

`vpnd/client.rs:445` — `vpn_connect`:

```rust
pub async fn vpn_connect(&self) -> Result<(), VpndError> {
    let mut vpnd = self.vpnd().await?;
    vpnd.connect_tunnel()   // <-- ConnectTunnel RPC, nothing else
        .or_else(async |e| self.handle_rpc_error("connect_tunnel", e).await)
        .await?;
    ...
}
```

**Zero setters fire at connect time on the Tauri client.** The daemon connects against whatever state happens to be loaded from prior interactions or from its own defaults.

The state is built up _opportunistically_ by individual UI commands:

- `set_vpn_mode` (`tunnel.rs:122-125`) — **only** pushes `set_two_hop(...)`. Nothing else.
- `set_node` (`tunnel.rs:127-139`) — pushes `set_entry_node` / `set_exit_node` when the user actively picks one in the UI.
- `ModeToggle.tsx:95-119` — when the user toggles Fast/Anonymous: first `applyAlgorithm('explicit')` (→ `set_gateway_selection_algorithm`), then `applyVpnMode(...)` (→ `set_two_hop`). Note: entry/exit are _not_ re-pushed during a mode toggle.

So Fast-mode connect on Tauri assumes:

- The user toggled the UI mode at some point this session, which means `set_two_hop(true)` and `set_gateway_selection_algorithm('explicit')` were pushed.
- The entry/exit nodes the user picked are still on the daemon.
- No other config field matters (or the daemon's defaults are fine).

When any of those assumptions breaks, you get a silent failure inside the daemon and the UI just times out / shows a generic error.

---

## 4. Other clients for context

- **iOS / macOS** (`nym-vpn-apple/`) — similar to Tauri: settings are pushed by the screen that owns the toggle (`ConnectionManager+Settings.swift`), and the connect path (`GRPCManager+Connection.swift:13-16` on macOS; XPC tunnel-provider message on iOS) does not re-sync. Same architectural shape as Tauri. iOS happens to _work_ in practice because its UX flow tends to push setters via small per-field invokes shortly before connect.
- **Android** is the outlier — it's the only client with a deliberate "re-assert the entire config before each connect" step.

---

## 4b. The actual default-value gap — `netstack`

**Daemon default** for `netstack` (from `nym-vpn-lib-types/src/service.rs:120` in the `Default` impl for `VpnServiceConfig`):

```rust
netstack: false,
```

**Tauri:** never calls `SetNetstack`. Grep for it across `nym-vpn-app/`:

```
$ grep -rn "set_netstack\|setNetstack\|SetNetstack" nym-vpn-app/src-tauri/src/ nym-vpn-app/src/
(no hits)
```

So Tauri's daemon stays at `netstack: false` for the entire session. The proto setter `SetNetstack(BoolValue)` exists (`nym_vpn_service.proto:1143`), there's just no Tauri client code wired to it.

**Android's runtime daemon config:** `netstack: true` (verified in the latest logcat, both Fast attempts at 23:16 and 23:18). Even though the Android UniFFI-side `VpnConfig` struct doesn't expose a `netstack` field (only `enableTwoHop`/`enableBridges`/etc., see `nym_vpn_lib_uniffi.kt:6240-6310`), the daemon-internal config that Android ends up with has `netstack: true`. That comes from somewhere in the UniFFI-vs-gRPC init path divergence and is worth a separate look — the symptom is what matters: Android Fast runs in netstack mode, Tauri Fast doesn't.

**What `netstack` does at runtime:** when on, the WireGuard `multihop_mode` is `Netstack` (see Android's logged `WireguardTunnelOptions { multihop_mode: Netstack }`). The second-hop wg connection runs over a user-space network stack (`netstack`) instead of through a kernel route. On Linux this matters because the alternate mode needs particular kernel/firewall capabilities or specific routing that may fail silently in a typical desktop environment.

**Suggested fix paths:**

- **a) Add `SetNetstack` to Tauri's client and default it to `true`.** Mirror Android's runtime config. Two-line Tauri Rust change + one frontend invoke at startup (or include in `sync_config` per §6). No daemon change, no proto change, fixes Fast on Linux.
- **b) Change the daemon's default for `netstack` in the gRPC init path to `true`.** Affects iOS/macOS too (which also don't currently call SetNetstack — same architectural gap as Tauri). Cleaner one-line change, but it shifts default behaviour for all gRPC clients at once and needs more scrutiny.
- **c) Expose a "Multihop mode" toggle in the Anti-censorship/Advanced settings.** Required if the choice is genuinely user-facing; otherwise (a) or (b) is enough.

(a) is the minimum-risk fix; (b) is cleaner architecturally but has wider blast radius.

## 5. Other concrete suspects, ranked

These remain as second-order causes even if QUIC bridges aren't the issue on your specific network. The daemon log lines from your failed attempt will discriminate.

1. **Stale entry/exit nodes from a prior session.** If you ran Mixnet earlier with explicit mixnet-only gateways picked, those are still on the daemon. Switching the mode toggle to Fast only pushes `set_two_hop(true)` + `set_gateway_selection_algorithm('explicit')` — it does **not** re-push or clear the gateways. The daemon will then try to bring up wg through a mixnet-only gateway and the connect will fail at the gateway-selection step.
2. **Algorithm mismatch.** The mode toggle force-sets the algorithm to `'explicit'`. If the user previously selected a country/region/specific gateway, that selection is now applied to the explicit algorithm in wg mode. If the underlying gateway doesn't have a healthy wg endpoint, the daemon fails to find a route. Android sends whichever algorithm `cfg.algorithm` resolves to (and the Android UI initialises it to `AUTO`, not `EXPLICIT`).
3. **Daemon never received an entry/exit at all.** If the user has never opened the entry/exit picker in this Tauri session, only `set_two_hop` and `set_gateway_selection_algorithm` have ever been pushed. Daemon is using its own fallback (Random) — that should work, but if Random picks a gateway with no wg endpoint, it fails.
4. **Race between toggle and connect.** `ModeToggle.tsx` fires two awaited invokes in sequence (`applyAlgorithm` then `applyVpnMode`) before the UI lets you press Connect — so this is less likely, but if a connect somehow fires while the second invoke is still in flight, the daemon sees an inconsistent state.
5. **Missing `setGatewayIndependence`.** Android sets `GatewayIndependence(differentNodeFamily=true, differentAsn=true)` at core init (`VpnCoreController.kt:359`). The gRPC proto does **not** currently expose a `SetGatewayIndependence` setter, so Tauri has no way to push this — the daemon uses its built-in default for non-Android clients. If the default permits same-ASN entry+exit pairs and your Fast mode requires the stricter rule, the daemon's pick is rejected. This is a daemon/proto gap, not a Tauri bug per se.

The log will tell us which. Typical signatures:

| Daemon log line                                             | Hits suspect |
| ----------------------------------------------------------- | ------------ |
| `No matching gateway found` / `failed to select wg gateway` | #1, #2, #3   |
| `gateway does not support wireguard`                        | #1           |
| `independence constraint violated` / `same ASN`             | #5           |
| `connect_tunnel called in wrong state`                      | #4           |

---

## 6. Fix (recommended)

Mirror Android's pattern: re-assert the user's full intent on every connect. The change is local to `src-tauri/` — no proto changes, no daemon changes.

### 6.1 Add a `sync_config` helper on `VpndClient`

```rust
// nym-vpn-app/src-tauri/src/vpnd/client.rs

/// Re-push the entire user-visible config to the daemon. Idempotent setters,
/// fail-fast: if any setter errors, abort and return the error.
#[instrument(skip_all)]
pub async fn sync_config(&self, cfg: &VpndConfig) -> Result<(), VpndError> {
    self.set_gateway_selection_algorithm(cfg.gateway_selection_algorithm).await?;
    self.set_two_hop(matches!(cfg.vpn_mode, VpnMode::Wg)).await?;
    self.set_entry_node(cfg.entry_node.clone()).await?;
    self.set_exit_node(cfg.exit_node.clone()).await?;
    self.set_enable_bridges(cfg.bridges).await?;
    self.set_fronting_mode(cfg.fronting_mode).await?;
    self.set_enable_custom_dns(cfg.custom_dns_enabled).await?;
    if cfg.custom_dns_enabled {
        self.set_custom_dns(cfg.custom_dns.clone()).await?;
    }
    self.set_enable_ad_blocking(cfg.ad_blocking_enabled).await?;
    self.set_allow_lan(cfg.allow_lan).await?;
    self.set_disable_ipv6(cfg.disable_ipv6).await?;
    self.set_mixnet_traffic_config(cfg.mixnet_traffic.clone()).await?;
    // Add others as they're added to VpndConfig — keep in sync with the
    // setters wired in src-tauri/src/vpnd/client.rs.
    Ok(())
}
```

The exact field names need cross-checking against `VpndConfig` (`src-tauri/src/vpnd/config/vpnd_config.rs`). The point is: **one helper that asserts every setter the UI cares about, called from one place.**

### 6.2 Call it from `connect`

```rust
// nym-vpn-app/src-tauri/src/commands/tunnel.rs:38
pub async fn connect(
    app: tauri::AppHandle,
    state: State<'_, SharedAppState>,
    vpnd: State<'_, VpndClient>,
) -> Result<TunnelState, BackendError> {
    // ... existing disconnect-state guard + Connecting transition ...

    let cfg = {
        let app_state = state.lock().await;
        app_state.vpnd_config.clone().ok_or_else(|| {
            BackendError::internal("no vpnd config available", None)
        })?
    };

    // Re-assert daemon state from the UI's canonical config. Matches Android's
    // applyCanonicalConfigToRustIfReady pattern; protects against stale daemon
    // state from prior sessions or other clients.
    if let Err(e) = vpnd.sync_config(&cfg).await {
        warn!("sync_config before connect failed: {e}");
        // Roll back the Connecting transition before bubbling
        let mut app_state = state.lock().await;
        app_state.tunnel = TunnelState::Disconnected;
        return Err(e.into());
    }

    app.emit_connecting();
    // ... existing vpnd.vpn_connect().await branch ...
}
```

### 6.3 Optional refinement (matches Android more precisely)

Cache the last-applied config in `AppState` and only push setters whose values changed. Saves daemon round-trips on the second+ connect of a session. Not necessary for correctness — the daemon's setters are idempotent. Skip this until profiling says it matters.

### 6.4 Simplify `set_vpn_mode`

`set_vpn_mode` in `tunnel.rs:122-125` currently just calls `set_two_hop`. Once `sync_config` runs at connect, the per-toggle setter is only needed if the user toggles in the UI without immediately connecting (to keep the daemon's "current mode" coherent for any other purpose). Keep it, but consider also pushing the gateway-selection algorithm here so the daemon's intermediate state stays sensible — currently that's done on the frontend (`ModeToggle.tsx`) which forks the logic across the language boundary.

---

## 7. Things to confirm next

- **Daemon log line** for the failing Fast connect. Once available, match against the table in §5 to pin the active suspect.
- **Repro on a fresh state.** Stop nymd, wipe its persistent state, restart, then in the Tauri app: do not touch anything else, switch to Fast, press Connect. If it fails clean, suspect #3 (default gateways picked by daemon don't expose wg). If it succeeds, the trigger is stale state — suspect #1 or #2.
- **Compare a Tauri-pushed entry/exit against an Android-pushed one.** If gateway identifiers (or their formats) differ between clients, the daemon may accept one and reject the other.
- **Should `SetGatewayIndependence` be added to the gRPC proto?** Android sets it. If the daemon's gRPC default is permissive, this is a latent inconsistency between Android (UniFFI clients) and gRPC clients (Tauri, iOS via Manager, macOS).

---

## Appendix A: Android log evidence (the actual config the daemon ran with)

From `~/Downloads/raw/logcat_1780002689448.txt` covering 23:16–23:19 on 2026-05-28.

**Daemon's `Using config: VpnServiceConfig {...}` dump** (logged at every state transition; values stable across the captured window):

```
VpnServiceConfig {
    entry_point: Country { two_letter_iso_country_code: "CH" },
    exit_point: Random,
    allow_lan: true,
    disable_ipv6: false,
    enable_two_hop: true,
    enable_bridges: false,          ← same as Tauri default
    enable_lewes_protocol: false,
    enable_ad_blocking: true,
    fronting_mode: OnRetry,         ← same as Tauri default
    netstack: true,                 ← the one Tauri doesn't set
    min_gateway_vpn_performance: None,
    residential_exit: false,
    enable_custom_dns: false,
    custom_dns: [],
    mixnet_traffic: MixnetTrafficConfig { ... },
    network_stats: NetworkStatisticsConfig { enabled: true, allow_disconnected: false },
    split_tunnel: SplitTunnelSettings { enabled: false, apps: [] },
    geo_exclusion: GeoExclusionSettings { enabled: false, listen_port: 1080, excluded_countries: ["CN"] },
    gateway_selection_algorithm_config: GatewaySelectionAlgorithmConfig {
        enable_geo_location: false,
        gateway_selection_algorithm: Auto,
    },
}
```

**Sample successful Fast connect with `fronting_mode: OnRetry` doing its job — first try direct, retry transparently upgrades to QUIC bridge:**

```
05-28 23:16:24.485  New tunnel state: Connecting wg [Gs5Nk3…] → [7tKwZyD…], resolving api addresses, try #0
…
05-28 23:16:30.910  New tunnel state: Connecting wg to 94.232.246.49:51822 [Gs5Nk3…] → 103.99.39.82:51822 [7tKwZyD…], connecting tunnel, try #0
05-28 23:16:33.454  Establishing DVPN QUIC transport tunnel                  ← OnRetry upgrade
05-28 23:16:33.512  quic transport connected in 58.202461ms
05-28 23:16:33.514  quic transport connected, udp forwarder open on 127.0.0.1:53610
05-28 23:16:33.593  New tunnel state: Connecting wg ... via bridge 94.232.246.49:4443, connecting tunnel, try #0
05-28 23:16:36.598  Tunnel connection is failing (retry: 1)
05-28 23:16:36.844  Tunnel connection is viable
05-28 23:16:36.845  New tunnel state: Connected wg ... via bridge 94.232.246.49:4443
```

**Second Fast connect 2 minutes later — same config, this time connected direct without bridge upgrade:**

```
05-28 23:18:07.564  New tunnel state: Connecting wg, refreshing gateways, try #0
05-28 23:18:09.379  New tunnel state: Connecting wg to 45.12.111.14:51822 [4KmPPM…] → 83.212.79.67:51822 [BnE33Pp…], connecting tunnel, try #0
05-28 23:18:12.418  Tunnel connection is failing (retry: 1)
05-28 23:18:12.529  Tunnel connection is viable
05-28 23:18:12.529  New tunnel state: Connected wg to 45.12.111.14:51822 ... → 83.212.79.67:51822 ...     ← no bridge!
```

So bridges are _not_ required for Fast to work — the daemon handles that transparently via `OnRetry`. The one config field that Android has and Tauri doesn't is `netstack: true`.

## 9. Mitigation: detect another active VPN before connect

Confirmed against a live failure: the `Connection refused` storm in the daemon log was caused by the user already being connected to another WireGuard VPN (`wg0` via NetworkManager). `ip route get 8.8.8.8 → 8.8.8.8 dev wg0 table 52020`. With nym-vpnd stopped, all the gateway / CDN endpoints were reachable through `wg0`. With nym-vpnd running, the daemon's firewall allow-rules apparently don't compose with a pre-existing tunnel as the default route, so everything outbound except LAN gets rejected.

That's a daemon-side bug (`nym-vpn-core/crates/nym-firewall/` — allow-rules need to be interface-aware about pre-existing tunnels, or interface-agnostic). But while that's being addressed, the Tauri client can **detect the situation pre-connect and warn the user** so they don't burn 30 minutes wondering why nothing works.

Target platforms: Linux + Windows (macOS/iOS/Android out of scope for this app). Implementation is small, fully read-only, no new heavy dependencies.

### 9.1 Detection logic

A VPN is "active" if **any** of:

1. The default route's output interface is a tunnel (link kind `wireguard`, `tun`, or `utun`).
2. Any interface other than `lo` / known bridges (`docker*`, `virbr*`, `br-*`) is up AND classified as a tunnel.
3. The interface name matches a well-known VPN brand pattern: `wg*`, `nordlynx*`, `mullvad*`, `proton*`, `tun*`, `tap*`, `ipsec*`, `nm-*` (NetworkManager VPNs).

Exclude **nym-vpnd's own tunnel** from the report (typical name `nymtun0` or `nymvpn` — confirm from daemon code). If unsure, only emit when the AppState's tunnel is `Disconnected` — we know our own tunnel isn't up.

### 9.2 Module layout

```
nym-vpn-app/src-tauri/src/sys/
    mod.rs
    vpn_detection.rs        ← new
        linux.rs            ← cfg(target_os = "linux")
        windows.rs          ← cfg(target_os = "windows")
```

### 9.3 Linux implementation (no new deps — uses `std::fs` only)

```rust
// sys/vpn_detection/linux.rs
use std::fs;

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export, export_to = "tauri.ts", rename = "TActiveVpn")]
pub struct ActiveVpn {
    pub interface: String,
    pub kind: VpnKind,
    pub is_default_route: bool,
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export, export_to = "tauri.ts", rename = "TVpnKind")]
#[serde(rename_all = "kebab-case")]
pub enum VpnKind {
    Wireguard,
    Tun,
    Mullvad,
    NordLynx,
    Proton,
    Other,
}

const OUR_TUNNEL_NAMES: &[&str] = &["nymtun0", "nymvpn", "nymvpn0"]; // confirm
const SAFE_BRIDGE_PREFIXES: &[&str] = &["docker", "virbr", "br-", "veth"];

pub fn detect() -> Vec<ActiveVpn> {
    let default_iface = default_route_iface();
    let Ok(entries) = fs::read_dir("/sys/class/net") else { return Vec::new(); };

    entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "lo"
                || OUR_TUNNEL_NAMES.contains(&name.as_str())
                || SAFE_BRIDGE_PREFIXES.iter().any(|p| name.starts_with(p))
            {
                return None;
            }
            let kind = classify(&name)?;
            Some(ActiveVpn {
                is_default_route: default_iface.as_deref() == Some(&name),
                interface: name,
                kind,
            })
        })
        .collect()
}

fn default_route_iface() -> Option<String> {
    let routes = fs::read_to_string("/proc/net/route").ok()?;
    for line in routes.lines().skip(1) {
        let mut parts = line.split_whitespace();
        let iface = parts.next()?;
        let dest = parts.next()?; // hex, big-endian-host-order
        if dest == "00000000" {
            return Some(iface.to_string());
        }
    }
    None
}

fn classify(name: &str) -> Option<VpnKind> {
    // Prefer the kernel's own link-kind from sysfs
    if let Ok(uevent) = fs::read_to_string(format!("/sys/class/net/{name}/uevent"))
        && uevent.contains("DEVTYPE=wireguard")
    {
        return Some(VpnKind::Wireguard);
    }

    // Fall back to name patterns
    match name {
        n if n.starts_with("wg")       => Some(VpnKind::Wireguard),
        n if n.starts_with("nordlynx") => Some(VpnKind::NordLynx),
        n if n.starts_with("mullvad")  => Some(VpnKind::Mullvad),
        n if n.starts_with("proton")   => Some(VpnKind::Proton),
        n if n.starts_with("tun") || n.starts_with("tap") => Some(VpnKind::Tun),
        _ => None,
    }
}
```

### 9.4 Windows implementation (sketch)

Use `windows-sys` crate's `GetIpForwardTable2` to find the default route's interface LUID, then `GetAdaptersAddresses` to get its description. Flag if description matches WireGuard/OpenVPN/Mullvad/Nord/Proton, or `IfType == IF_TYPE_TUNNEL`. About 40 lines; the `windows` Cargo crate is already widely used in the workspace.

(macOS would use `route -n get default` + check link-type via `getifaddrs` + sysctl — but Tauri doesn't ship on macOS, this app is Linux + Windows only.)

### 9.5 Tauri command + integration

```rust
// sys/vpn_detection/mod.rs
pub use linux::ActiveVpn;
#[cfg(target_os = "linux")] mod linux;
#[cfg(target_os = "windows")] mod windows;

pub fn detect_active_vpns() -> Vec<ActiveVpn> {
    #[cfg(target_os = "linux")] { linux::detect() }
    #[cfg(target_os = "windows")] { windows::detect() }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))] { Vec::new() }
}
```

```rust
// commands/sys.rs (new command)
#[tauri::command]
pub fn detect_active_vpns() -> Vec<ActiveVpn> {
    crate::sys::vpn_detection::detect_active_vpns()
}
```

Wire into `connect` in `commands/tunnel.rs:38-93` BEFORE the Connecting transition:

```rust
let other_vpns = crate::sys::vpn_detection::detect_active_vpns();
if !other_vpns.is_empty() {
    // Don't auto-fail — the user may have a tun-mode SSH session that isn't
    // actually a VPN. Return a typed warning that the frontend can present as
    // a dialog with "Connect anyway" + "Disconnect other VPN first" buttons.
    return Err(BackendError::with_data(
        "another VPN is active",
        ErrorKey::AnotherVpnActive,
        json!({ "vpns": other_vpns }),
    ));
}
```

Add `ErrorKey::AnotherVpnActive` to `src-tauri/src/error.rs`. ts-rs will regen the TS enum.

### 9.6 Frontend UX

Two layers:

1. **Pre-connect dialog** when `AnotherVpnActive` is returned: list the interfaces + kinds, "We detected `wg0` (WireGuard) as your active VPN. NymVPN may fail to connect through another tunnel. Disconnect it first or try anyway." Buttons: "Cancel", "Try anyway" (re-invoke `connect` with a `force=true` flag that skips the check).
2. **Passive banner on Home** when tunnel is Disconnected and `detect_active_vpns()` returns non-empty. Subtle, dismissible per-session. Polled on Home mount + on `connection-state` events.

The first one is the gating UX; the second is just a heads-up.

### 9.7 Caveats

- Some user setups (Tailscale, ZeroTier, corporate split-tunnel VPNs) legitimately coexist with another VPN. The "Try anyway" button respects that.
- A "tun0" interface that's actually an SSH tunnel will get flagged. Cost: one extra dialog. Acceptable.
- Periodic polling for the passive banner is fine — `/proc/net/route` and `/sys/class/net` are cheap reads.

---

## Appendix B: file pointers

- Tauri connect command — `nym-vpn-app/src-tauri/src/commands/tunnel.rs:38-93`
- Tauri mode toggle command — `nym-vpn-app/src-tauri/src/commands/tunnel.rs:122-125`
- Tauri vpnd client `vpn_connect` — `nym-vpn-app/src-tauri/src/vpnd/client.rs:445`
- Tauri vpnd client setters — `nym-vpn-app/src-tauri/src/vpnd/client.rs` (`set_two_hop`, `set_entry_node`, `set_exit_node`, `set_gateway_selection_algorithm`, …)
- Tauri mode-toggle frontend — `nym-vpn-app/src/screens/home/ModeToggle.tsx:60-119`
- Android `connectLocked` — `nym-vpn-android/core/src/main/java/net/nymtech/vpn/backend/controller/VpnCoreController.kt:163-206`
- Android `applyCanonicalConfigToRustIfReady` — same file, `:370-422`
- Android initial config (gatewayIndependence etc.) — same file, `:342-360`
- Proto setters and `ConnectTunnel` — `nym-vpn-core/crates/nym-vpn-proto/proto/nym_vpn_service.proto:1136-1172`
