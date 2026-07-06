## Why

Today, writing NymVPN **app** logs to disk requires launching the app with the `--log-file` CLI flag (or the `LOG_FILE` env var) — something a normal user never does, so app logs are effectively unavailable when support needs them. Users should be able to turn debug logging on and off from the app's settings, at runtime, without relaunching or restarting the tunnel.

## What Changes

- **BREAKING**: Remove the `--log-file` (`-l`) CLI argument. It no longer controls app logging.
- **BREAKING**: Remove the `LOG_FILE` environment variable (`ENV_LOG_FILE`) completely. It no longer controls app logging.
- Add a persisted **"Enable debug logging"** preference, stored in the app config file (`config.toml`), defaulting to **off**. This becomes the single source of truth for app file logging.
- Add a **switch** titled _"Enable debug logging"_ to Settings → Data, privacy & logs (the `/settings/data-privacy/logs` screen), controlling **app** logs only (not daemon logs).
- Make file logging **runtime-toggleable**: enabling starts writing `app.log` immediately; disabling stops writing immediately and leaves no active log file — all without restarting the app, its gRPC connection, or the VPN tunnel.
- Add Tauri commands (in `commands/log.rs`) to set and read the debug-logging preference, and wire the switch into frontend state initialization (mirroring the existing Sentry/network-stats toggles).

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `diagnostics-logging`: Application logging setup is no longer gated by the `--log-file` CLI flag or the `LOG_FILE` env var; it is gated solely by a persisted preference and can be toggled at runtime via a settings switch and backend commands.
- `system-integration`: CLI argument parsing no longer includes `log-file`.

## Impact

- **Rust backend**:
  - `src-tauri/src/cli.rs` — remove the `log_file` field from `Cli`.
  - `src-tauri/src/log.rs` — build the tracing subscriber with a reload-able file layer; read the persisted preference for the initial state; expose a runtime toggle; remove the `ENV_LOG_FILE` constant and its `env::is_truthy` check.
  - `src-tauri/src/fs/config.rs` — add a `debug_logging` field to `AppConfig`.
  - `src-tauri/src/state/app.rs` — hold the reload handle + non-blocking writer guard so commands can toggle at runtime.
  - `src-tauri/src/commands/log.rs` — new `set_debug_logging` / `debug_logging_enabled` commands, registered in `main.rs`.
- **Generated types**: `src/types/tauri.ts` regenerated via `npm run tsgen` (the `logFile` field on `Cli` disappears).
- **Frontend**:
  - `src/screens/settings/logs/Logs.tsx` — add the "Enable debug logging" switch.
  - `src/store` main slice + `src/types/app-state.ts` — add `debugLogging` state and a `set-debug-logging` action.
  - `src/state/init.ts` — initialize the toggle from `debug_logging_enabled`.
  - `src/i18n/**/settings.json` — add the new label/description strings.
- **Dependencies**: none added; uses `tracing-subscriber`'s built-in `reload` module (already in the tree at 0.3.23).
