# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this project is

A Tauri 2 desktop VPN client (NymVPN) for Windows and Linux. The frontend is React 19 + TypeScript + Tailwind CSS v4; the backend is Rust. The Tauri app is a thin shell — the actual VPN work is done by a separate daemon (`nym-vpnd`) that this app communicates with via gRPC.

Current version: `1.30.0-beta`. Minimum compatible daemon version: `>=1.30.0-beta` (see `vpnd_compat.toml`).

## Commands

### Development

```sh
npm run dev:app       # Full Tauri app (requires Rust toolchain)
npm run dev           # Vite dev server only (no Tauri)
```

### Build

```sh
npm run build         # Frontend only (tsc + vite build → dist/)
npm run build:app     # Full desktop app with Tauri
```

### Checks (run before committing)

```sh
npm run check         # lint + tscheck + fmt:check (all three)
npm run lint          # ESLint on src/
npm run lint:fix      # Auto-fix ESLint errors
npm run fmt           # Prettier format
npm run tscheck       # TypeScript type-check (no emit)
```

### Rust backend

```sh
cargo build                         # Build Rust backend (from src-tauri/)
cargo test                          # Run Rust tests (also runs ts-rs type generation)
cargo +nightly clippy -- -Dwarnings # Lint Rust code
cargo +nightly fmt                  # Format Rust code
```

### Type generation (Rust → TypeScript)

```sh
npm run tsgen         # cargo test + prettier → regenerates src/types/tauri.ts
```

Run this whenever you add or modify Rust types annotated with `#[derive(ts_rs::TS)]`.

### License generation

```sh
npm run gen:licenses  # Regenerates both JS and Rust license files
```

## Architecture

### Two-process model

The app has two separate processes that must both be running for full functionality:

1. **nym-vpnd** — system daemon that handles actual VPN connections (external binary, not in this repo)
2. **nym-vpn-app** — this Tauri app (the UI + control layer)

The Tauri backend (`src-tauri/src/vpnd/client.rs`) maintains a gRPC connection to nym-vpnd. The frontend communicates with the Tauri backend via Tauri IPC commands and events.

### Frontend → Backend communication

- **Tauri commands** (`invoke()`): request/response calls from JS to Rust, defined in `src-tauri/src/commands/`
- **Tauri events**: push notifications from Rust to JS, defined in `src-tauri/src/events.rs`
- All TypeScript types for IPC are **auto-generated** from Rust structs in `src/types/tauri.ts` — never edit this file manually

### Frontend state management

Main app state is managed via **Zustand** stores in `src/store/`:

- `src/store/slices/createMainSlice.ts` — tunnel state, account, VPN mode, config, feature flags, mixnet config, split-tunnel settings
- `src/store/slices/createSocks5Slice.ts` — SOCKS5 proxy config
- `src/store/slices/gateways/` — gateway selection state
- `src/store/nodeListState.ts` — node list

Zustand stores are accessed via React Contexts in `src/contexts/`:

- `src/contexts/main/` — wraps the main Zustand store
- `src/contexts/gateways/` — gateway selection/caching
- `src/contexts/socks5/` — SOCKS5 proxy
- `src/contexts/dialog/` — global dialog management
- `src/contexts/tray/` — system tray state
- `src/contexts/topbar/` — top bar UI state
- `src/contexts/autologin/` — auto-login after web account creation (PIN code dialog)

All providers are composed in `src/App.tsx`.

### Routing

React Router 7, configured in `src/router.tsx`. Route constants are in `src/types/routes.ts`. The Home screen is lazy-loaded behind a Suspense boundary.

Key routes:

- `/` → startup gate (routes to home/login based on auth state)
- `/home` → main VPN control screen
- `/welcome` → welcome screen
- `/login`, `/signup` → authentication
- `/account`, `/account/select-a-plan` → account management
- `/settings/**` → settings (see settings section below)
- `/entry-node-location`, `/exit-node-location`, `/node-details` → node selection
- `/hideout/onboarding` → onboarding carousel

### Settings screens

All settings routes live under `/settings`:

| Route                               | Screen                                                                 |
| ----------------------------------- | ---------------------------------------------------------------------- |
| `/settings`                         | Settings index                                                         |
| `/settings/account`                 | Account management                                                     |
| `/settings/appearance`              | Theme & display                                                        |
| `/settings/appearance/display`      | UI scaling                                                             |
| `/settings/appearance/lang`         | Language selector                                                      |
| `/settings/dns`                     | Custom DNS servers                                                     |
| `/settings/anti-censorship`         | Domain fronting toggle (+ Lewes protocol in DEV)                       |
| `/settings/socks5`                  | SOCKS5 proxy                                                           |
| `/settings/mixnet-tuning`           | Mixnet traffic tuning (mixing delays, continuous traffic, performance) |
| `/settings/split-tunneling`         | Per-app VPN bypass (Windows)                                           |
| `/settings/advanced-settings`       | Advanced settings                                                      |
| `/settings/data-privacy`            | Data & privacy                                                         |
| `/settings/data-privacy/logs`       | App log viewer                                                         |
| `/settings/data-privacy/diagnostic` | Diagnostic info                                                        |
| `/settings/support`                 | Support links                                                          |
| `/settings/legal`                   | Legal info & licenses                                                  |
| `/settings/dev`                     | Developer features (network env, Lewes protocol)                       |

### Tauri backend module layout

```
src-tauri/src/
  commands/          # IPC handlers exposed to frontend (one file per domain)
    account.rs       # Account operations (login, logout, link social)
    cli.rs           # CLI argument passing
    daemon.rs        # Daemon status and control
    db.rs            # Database read/write
    diagnostic.rs    # Diagnostic data collection
    fs.rs            # File system operations
    gateway.rs       # Gateway discovery and selection
    log.rs           # Logging configuration
    network_stats.rs # Network bandwidth/latency stats
    sentry.rs        # Sentry integration
    socks5.rs        # SOCKS5 proxy configuration
    sys.rs           # System information
    tray.rs          # Tray icon/menu
    tunnel.rs        # VPN tunnel control (connect, disconnect, mode, fronting mode)
    updater.rs       # App update checking/installation
    window.rs        # Window management
  vpnd/              # gRPC client + response mapping for nym-vpnd daemon
    client.rs        # gRPC connection management
    account.rs       # Account RPC calls
    account_links.rs # Account linking RPC calls
    config/          # Daemon config structures (mixnet_config.rs, vpnd_config.rs)
    deeplink.rs      # Deep link handling
    diagnostic.rs    # Diagnostic data RPC calls
    feature_flags.rs # Feature flag parsing (quic, domain_fronting, zknym_credential, mixnet_tuning)
    gateway.rs       # Gateway RPC data
    node.rs          # Node RPC data
    socks5.rs        # SOCKS5 proxy RPC calls
    tunnel.rs        # Tunnel control (FrontingMode: Off | OnRetry | Always)
    vpnd_status.rs   # Daemon status types
  fs/                # File system helpers, config paths, Windows app discovery
    app_discovery/windows_discovery.rs  # App enumeration for split tunneling
  state/             # Shared app state (Arc<Mutex<AppState>>)
  tray.rs            # System tray icon/menu
  window.rs          # Main + error window management
  db.rs              # Sled embedded key-value store
  updater.rs         # App updater logic
  icon_extractor.rs  # Windows app icon extraction
```

### Feature flags

Feature flags are reported by the daemon and exposed to the frontend via `FeatureFlags` in `src/types/tauri.ts`:

- `quic` — QUIC protocol support
- `domain_fronting` — domain fronting / stealth API (FrontingMode toggle in anti-censorship settings)
- `zknym_credential` — zero-knowledge credential mode
- `mixnet_tuning` — mixnet traffic tuning enabled

### Platform-specific code

- Windows-specific: `src-tauri/src/fs/app_discovery/windows_discovery.rs` (split tunneling app discovery), `src-tauri/src/icon_extractor.rs`, `src-tauri/src/updater.rs`
- Tauri platform overrides: `tauri.windows.conf.json` for Windows-specific bundle/window settings
- Feature flags in `Cargo.toml` gate Windows vs Linux code paths

### Type generation workflow

Rust types annotated with `#[derive(Serialize, ts_rs::TS)]` are exported via `cargo test` to `src/types/tauri.ts`. When modifying types on the Rust side that cross the IPC boundary, always run `npm run tsgen` afterward.

## Key conventions

- **Prettier**: 2-space indent, single quotes, semicolons, LF line endings
- **Imports**: ESLint enforces ordering (external → internal → relative). Run `npm run lint:fix` to auto-sort.
- **CSS**: Tailwind CSS v4 utility classes with custom CSS properties for theming. Theme tokens (colors, breakpoints) are defined in `src/styles.css`.
- **i18n**: All user-visible strings go through `useTranslation()` from i18next. Translation files are in `src/i18n/`. 14 active locales including RTL (Arabic, Persian). 17 namespaces. Never hardcode user-visible strings.
- **Rust formatting**: `cargo fmt` with default settings; clippy must pass without warnings.
- **Auto-generated files**: Never manually edit `src/types/tauri.ts` — regenerate with `npm run tsgen`.

## Environment variables

Copy `.env.sample` to `.env` to configure:

- `APP_SENTRY_DSN` — Sentry error reporting DSN (leave blank to disable)
- `APP_NOSPLASH` — set to `true` to skip the intro splash screen
