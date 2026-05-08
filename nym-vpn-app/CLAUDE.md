# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this project is

A Tauri 2 desktop VPN client (NymVPN) for Windows and Linux. The frontend is React 19 + TypeScript + Tailwind CSS v4; the backend is Rust. The Tauri app is a thin shell — the actual VPN work is done by a separate daemon (`nym-vpnd`) that this app communicates with via gRPC.

## Commands

### Development

```sh
npm run dev:app       # Full Tauri app (requires Rust toolchain)
npm run dev:browser   # Frontend only in the browser with Tauri command mocks
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

No Redux or Zustand — state is managed through nested React Contexts in `src/contexts/`. The main app context is in `src/contexts/main/`. Each domain (gateways, node-list, socks5, tray, dialog, etc.) has its own context provider. All providers are composed in `src/App.tsx`.

### Routing

React Router 7, configured in `src/router.tsx`. Route constants are in `src/types/routes.ts`. The Home screen is lazy-loaded behind a Suspense boundary.

### Tauri backend module layout

```
src-tauri/src/
  commands/     # IPC handlers exposed to frontend (one file per domain)
  vpnd/         # gRPC client + response mapping for nym-vpnd daemon
  fs/           # File system helpers, config paths, Windows app discovery
  state/        # Shared app state (Arc<Mutex<AppState>>)
  tray.rs       # System tray icon/menu
  window.rs     # Main + error window management
  db.rs         # Sled embedded key-value store
```

### Platform-specific code

- Windows-specific features live in `src-tauri/src/fs/app_discovery/windows_discovery.rs` (app discovery for split tunneling), `src-tauri/src/icon_extractor.rs`, and `src-tauri/src/updater.rs`
- Tauri platform overrides: `tauri.windows.conf.json` for Windows-specific bundle/window settings
- Feature flags in `Cargo.toml` gate Windows vs Linux code paths

### Type generation workflow

Rust types annotated with `#[derive(Serialize, ts_rs::TS)]` are exported via `cargo test` to `src/types/tauri.ts`. When modifying types on the Rust side that cross the IPC boundary, always run `npm run tsgen` afterward.

### Browser dev mode

`npm run dev:browser` runs the frontend without a Tauri runtime. Mock implementations of all Tauri commands are in `src/dev/tauri-cmd-mocks/`. This is the fastest way to iterate on UI changes.

## Key conventions

- **Prettier**: 2-space indent, single quotes, semicolons, LF line endings
- **Imports**: ESLint enforces ordering (external → internal → relative). Run `npm run lint:fix` to auto-sort.
- **CSS**: Tailwind utility classes with custom CSS properties for theming. Theme tokens are defined in `src/styles.css`.
- **i18n**: All user-visible strings go through `useTranslation()` from i18next. Translation files are in `src/i18n/`.
- **Rust formatting**: `cargo fmt` with default settings; clippy must pass without warnings.

## Environment variables

Copy `.env.sample` to `.env` to configure:

- `VITE_SENTRY_DSN` — Sentry error reporting (leave blank to disable)
- `VITE_SHOW_SPLASH_SCREEN` — Toggle intro animation
