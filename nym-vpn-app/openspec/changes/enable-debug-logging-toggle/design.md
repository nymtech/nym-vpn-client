## Context

App logging is set up once in `main()` at `src-tauri/src/log.rs::setup_tracing`, called very early (`main.rs:99`) before the Tauri builder runs. Today it writes `app.log` only when `cli.log_file || env::is_truthy("LOG_FILE")` is true (`log.rs:59`). Both of those controls are being removed. The `tracing` subscriber is a **global, initialized exactly once** via `.init()` — which is why the sibling Sentry toggle (`commands/sentry.rs`) persists a preference but requires an app restart to take effect.

We want app file logging to be user-controlled from settings **and toggleable at runtime** (no restart). Unlike Sentry's global guard, `tracing-subscriber` (0.3.23, already in the tree) ships a `reload` module built exactly for hot-swapping a layer inside a running subscriber, so genuine runtime toggling is achievable.

Precedents to mirror:

- Persisted preference in `AppConfig` / `config.toml`, read early via `AppConfig::read()` (`config.rs`, used for `sentry_monitoring` at `main.rs:95`).
- Runtime state held in `AppState` (`state/app.rs`), like `sentry_client`.
- Enable/disable + getter commands, like `enable_sentry` / `disable_sentry` / `sentry_enabled`.
- Frontend init via `src/state/init.ts` (`invoke('sentry_enabled') → dispatch`) and a Zustand slice action like `set-monitoring`.

## Goals / Non-Goals

**Goals:**

- Remove the `--log-file` CLI flag **and** the `LOG_FILE` env var as controls for app logging.
- A single source of truth: a persisted, default-off "Enable debug logging" preference in `config.toml`.
- Runtime enable/disable of app file logging with no app/tunnel restart.
- When disabled, no active log file is being written.

**Non-Goals:**

- Daemon (`nym-vpnd`) logs — entirely out of scope; this is app logs only.
- Deleting existing log files on disable (a separate `delete_app_logs` command already exists).
- Changing the log level / `EnvFilter` behavior, stdout logging, or the Sentry layer.
- Runtime-configurable log verbosity level (the `-L/--log-level` arg stays as-is).

## Decisions

### Decision 1: Runtime reload of an `Option<FileLayer>`, not restart-required

Build the registry with a `reload::Layer` wrapping an `Option<FileLayer>`:

```
registry()
  .with(env_filter)
  .with(stdout_layer)
  .with(reload_layer)   // reload::Layer<Option<FileLayer>, S>
  .with(sentry_layer?)
  .init()               // still once; never re-initialized
```

- **Enable**: construct a fresh `tracing_appender::rolling::never` appender + `non_blocking` writer (this is the moment `app.log` is created), build the compact file `fmt::layer`, `handle.reload(Some(layer))`, and store the returned `WorkerGuard` in `AppState`.
- **Disable**: `handle.reload(None)` and drop the stored guard (flushes + stops the appender's worker thread).

Swapping `Option<FileLayer>` (not a filter level) means **no file exists on disk while disabled**, satisfying "prevent logs from being saved." Log rotation (rotate `app.log` → `app.old.log`) runs at the moment of enable.

_Why not alternatives:_

- **Restart-required config flag (Sentry clone):** simplest, but fails the explicit "toggle at runtime" requirement.
- **Reload only a `LevelFilter` on an always-present file layer:** the appender would create/hold `app.log` even when "off", leaving an (empty) file on disk — violates the disable semantics.

### Decision 2: Type-erase the reload handle behind a closure in `AppState`

`reload::Handle<L, S>` names both the layer type and the entire subscriber stack `S`, which is verbose and brittle to store. Inside `setup_tracing` (where `S` is already inferred) build a closure that captures the handle + guard slot and exposes a simple boolean API, e.g. store in `AppState`:

```
debug_logging: Mutex<DebugLoggingControl>   // { set: Box<dyn Fn(bool) -> Result<()> + Send + Sync>, guard: Option<WorkerGuard> }
```

Commands then call `set(true/false)` without naming any `tracing` types. `setup_tracing` returns this control object (replacing today's bare `Option<WorkerGuard>` return) so `main.rs` can `app.manage` it (or fold it into `AppState`).

### Decision 3: Effective-state resolution (single source of truth)

The persisted preference is the only control; the `--log-file` CLI flag and the `LOG_FILE` env var (including the `ENV_LOG_FILE` constant and its `env::is_truthy` check) are removed.

```
initial_enabled   = config.debug_logging                  // read once at startup
```

- At startup, file logging starts on iff `initial_enabled`.
- `set_debug_logging(enabled)` persists `enabled` to `config.toml` and applies it via the reload control.
- `debug_logging_enabled()` returns the current state.

### Decision 4: Persist in `AppConfig`, not the sled DB

`setup_tracing` runs before the sled `Db` is opened, and `AppConfig::read()` is already used at that early point for `sentry_monitoring`. Add `debug_logging: bool` (`#[serde(default)]`) to `AppConfig`, keeping startup-read symmetry with Sentry and avoiding a DB dependency in early init.

### Decision 5: Command placement & frontend wiring

- Commands `set_debug_logging(enabled: bool)` and `debug_logging_enabled() -> bool` live in `src-tauri/src/commands/log.rs` (alongside `log_js`) and are registered in `main.rs`'s `invoke_handler`.
- Frontend: `debugLogging` boolean on the main Zustand slice + `set-debug-logging` action (`types/app-state.ts`); initialized in `state/init.ts` via `invoke('debug_logging_enabled')`; the switch in `screens/settings/logs/Logs.tsx` optimistically dispatches then invokes `set_debug_logging`, matching the netstats/monitoring pattern in `DataAndPrivacy.tsx`.

## Risks / Trade-offs

- **Per-event `RwLock` read from `reload::Layer`** → Negligible for a desktop VPN UI's log volume; this is the sanctioned mechanism.
- **`WorkerGuard` lifecycle bug drops logs** (guard dropped too early = lost/incomplete file) → Guard is owned by `AppState` for the whole enabled period; disable is the only place it's dropped, right after `reload(None)`.
- **Removing `--log-file` and `LOG_FILE` breaks scripts/muscle-memory that pass them** → BREAKING and intentional; CLI parsing now errors on the unknown flag and the env var is ignored. There is no longer a headless/dev escape hatch — file logging must be enabled via settings (or by pre-seeding `config.toml`). Regenerate `src/types/tauri.ts` so the stale `logFile` field is removed.
- **`Cli` no longer used by `setup_tracing` for logging** → keep the `cli.log_level` handling; only the `log_file` branch is removed.

## Migration Plan

1. Add `debug_logging` to `AppConfig` (defaults false; existing configs deserialize fine via `#[serde(default)]`).
2. Rework `setup_tracing` to the reload architecture and return the control object.
3. Remove `Cli::log_file`; remove the `ENV_LOG_FILE` constant and its env check.
4. Add commands + register them; `npm run tsgen`.
5. Wire frontend state + settings switch + i18n strings.
6. No data migration needed; first run after upgrade defaults to logging off.

Rollback: revert the change; old configs ignore the unknown `debug_logging` key.

## Open Questions

_None outstanding._ The `LOG_FILE` env var is removed (single source of truth is the persisted preference), and the two commands live in `commands/log.rs`.
