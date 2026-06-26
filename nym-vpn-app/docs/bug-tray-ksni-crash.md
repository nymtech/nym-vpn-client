# Linux tray regression since #5384: app crashes on launch, and the tray icon no longer shows on XEmbed bars (i3)

## Summary

PR #5384 switched the Linux system tray from the Tauri-native (libayatana-appindicator) backend to `ksni`. This caused **two regressions** on Linux desktops that do not run an SNI `StatusNotifierWatcher` (e.g. i3 with i3bar, sway without an SNI tray, minimal GNOME without the AppIndicator extension):

1. **Crash on launch** — the tray-spawn error is propagated out of Tauri's `setup` hook and panics the whole app before the window opens.
2. **Loss of the tray icon** — even with the crash handled, `ksni` is SNI-only and cannot display on XEmbed (`_NET_SYSTEM_TRAY`) trays like i3bar. In `2026.10` the icon **did** appear in i3's tray.

Both behaviors worked in `2026.10`. The objective of the fix is to **restore the tray icon on XEmbed bars** (and not crash) — not merely to fail silently with no icon.

## Severity

High — the app crashes before the window opens for affected users, and the intended fix must also restore a previously-working feature (the tray icon), not just suppress the crash.

## Environment

- App version: `2026.11.0-beta.1` (also reproduces on nightly `nym-vpn-nightly-20260619054702`)
- OS: Ubuntu 24.04, kernel 6.17
- Desktop: i3 (X11); tray provided by i3bar (XEmbed `_NET_SYSTEM_TRAY` protocol)
- `busctl --user list | grep -i statusnotifier` → no `org.kde.StatusNotifierWatcher` registered
- `libayatana-appindicator3.so.1` present (the lib the old backend used)

## Steps to reproduce

1. On a Linux session with no SNI watcher (e.g. i3 + i3bar tray):
   ```sh
   busctl --user list | grep -i statusnotifier   # prints nothing
   ```
2. Launch the app: `./nym-vpn-app`

## Actual behavior

```
ERROR nym_vpn_app::tray::linux: failed to spawn ksni tray: Watcher(ServiceUnknown("The name org.kde.StatusNotifierWatcher was not provided by any .service files"))

thread 'main' panicked at tauri-2.11.1/src/app.rs:1417:11:
Failed to setup app: error encountered during setup hook: failed to spawn ksni tray: Watcher(ServiceUnknown(...))
```

The process aborts; the window never appears. (Once the crash is handled, the icon is simply absent — see regression #2.)

## Expected behavior

Same as `2026.10`: the app launches, and a tray icon appears in the i3/XEmbed tray. No SNI watcher should be required.

## Root cause

PR **#5384** ("feat(tray): toggle window on left-click, menu on right-click", commit `37347bd8c`, 2026-06-12) replaced the Tauri `tray-icon` backend with `ksni` on Linux, to obtain `activate`/`secondary_activate` callbacks (left-click toggles the window). It introduced the `#[cfg(target_os = "linux")]` tray split; before it, all platforms shared the Tauri-native `TrayIconBuilder` implementation.

- **Old backend (Tauri / libayatana-appindicator):** tries SNI first, and **falls back to a GtkStatusIcon using the XEmbed `_NET_SYSTEM_TRAY` protocol** when no `StatusNotifierWatcher` is present. i3bar implements `_NET_SYSTEM_TRAY`, so the icon rendered, and `TrayIconBuilder::build()` did not error → no crash.
- **New backend (`ksni`):** implements **only** StatusNotifierItem over D-Bus, with **no XEmbed fallback**. `ksni::Tray::spawn()` returns `Err(Watcher(ServiceUnknown(..)))` when no watcher exists, so on XEmbed-only desktops there is no way to show the icon at all.

The crash specifically comes from `TrayManager::new()` propagating that error at the call site in `main.rs`:

```rust
let tray_manager = TrayManager::new(app.handle())?;   // `?` inside setup() → fatal panic
```

`#5384` is **not** in `release/2026.10-tatry`; it only landed on the 2026.11/nightly line — which is the regression boundary.

## Proposed fix

The fix must restore the icon on XEmbed bars, not just stop the crash. Options, in order of preference:

1. **Hybrid backend (recommended):** keep `ksni` when an SNI `StatusNotifierWatcher` is present (preserves #5384's left-click toggle on SNI desktops), and **fall back to the Tauri-native / libayatana backend** (which provides the XEmbed GtkStatusIcon fallback) when no watcher exists. This matches `2026.10` behavior on i3 while keeping the new UX where it works.
2. **Revert the Linux tray to the Tauri-native backend.** Simplest and restores the icon everywhere it worked in `2026.10`, but loses #5384's Linux left-click-toggle improvement.

In all cases, make tray init **non-fatal**: never propagate a tray error out of the `setup` hook (a missing tray must not abort startup).

## Workaround (current beta)

Run an SNI bridge that registers a `StatusNotifierWatcher` and proxies SNI items into the XEmbed tray — e.g. `snixembed` (`exec --no-startup-id snixembed &` in the i3 config). This restores the icon in i3bar. (GNOME users: enable an AppIndicator extension.)

## Notes

A local change currently makes init non-fatal but degrades to *no icon* — that addresses only regression #1 and is therefore not a complete fix on its own; it should be combined with option 1 or 2 above to restore the icon.
