## 1. Backend — persistence & config

- [x] 1.1 Add `debug_logging: bool` (with `#[serde(default)]`) to `AppConfig` in `src-tauri/src/fs/config.rs`
- [x] 1.2 Remove the `log_file` field (and its `-l/--log-file` arg) from `Cli` in `src-tauri/src/cli.rs`
- [x] 1.3 Remove the `ENV_LOG_FILE` constant and its `env::is_truthy` usage from `src-tauri/src/log.rs`

## 2. Backend — reloadable tracing

- [x] 2.1 In `src-tauri/src/log.rs`, replace the static file-layer branch with a `reload::Layer` wrapping `Option<Box<dyn Layer>>`; set `initial_enabled = config.debug_logging`
- [x] 2.2 Extract file-layer construction (appender + `non_blocking` writer + rotation) into the runtime `apply` closure reused by startup and runtime enable
- [x] 2.3 Define `DebugLogging` (closure `apply: Fn(bool)` capturing the reload handle + `Option<WorkerGuard>` slot) and have `setup_tracing` return it
- [x] 2.4 Store the control as a dedicated managed `Mutex<DebugLogging>` (`SharedDebugLogging` in `state/mod.rs`) and `app.manage` it in `main.rs` (replaces the bare `_guard`). Note: kept separate from `AppState` rather than folded in, since `AppState` derives `Debug`/`Default` and the closure/guard don't fit that cleanly — same shared-state spirit as the design.
- [x] 2.5 Enable rotates the previous log file and creates a fresh `app.log`; disable flushes + drops the guard and leaves no active file

## 3. Backend — commands

- [x] 3.1 In `src-tauri/src/commands/log.rs`, add `set_debug_logging(enabled: bool)` command: persist to `AppConfig` and apply via the control
- [x] 3.2 In `src-tauri/src/commands/log.rs`, add `debug_logging_enabled() -> bool` command returning the current state
- [x] 3.3 Register both commands in `main.rs` `invoke_handler`

## 4. Type generation

- [x] 4.1 Run `npm run tsgen`; verified `logFile` is removed from `Cli` (commands take primitive args, so no new types needed)

## 5. Frontend — state

- [x] 5.1 Add `debugLogging: boolean` to `src/types/app-state.ts`, a `set-debug-logging` action + reducer + default to the main Zustand slice, and expose it in the `useMainState` selector (`store/index.ts`)
- [x] 5.2 Initialize `debugLogging` in `src/state/init.ts` via `invoke('debug_logging_enabled')`

## 6. Frontend — settings UI

- [x] 6.1 Add the "Enable debug logging" switch to `src/screens/settings/logs/Logs.tsx` (optimistic dispatch + `invoke('set_debug_logging', { enabled })`, matching the `Notifications.tsx` pattern)
- [x] 6.2 Add i18n strings (label + description) for the switch to `src/i18n/en/settings.json`

## 7. Verification

- [x] 7.1 `cargo +nightly clippy -- -Dwarnings` and `cargo build` pass; `cargo +nightly fmt`
- [x] 7.2 `npm run check` (lint + tscheck + fmt:check) passes
- [ ] 7.3 Manual: toggle on → `app.log` appears and receives new events; toggle off → writing stops and no active file; tunnel stays connected across both; `--log-file` now errors and `LOG_FILE=1` has no effect
