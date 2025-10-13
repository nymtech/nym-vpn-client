# AI coding agents: project guardrails and workflows

-   Repo layout (top-level):

    -   `nym-vpn-core/` Rust workspace (edition 2024). Core crates include daemon `nym-vpnd`, CLI `nym-vpnc`, proto `nym-vpn-proto`, RPC bindings, firewall/DNS, and shared types.
    -   `nym-vpn-app/` Desktop app (Tauri 2.x + React/Vite/TypeScript). Rust backend in `src-tauri/` with commands; TS frontend in `src/`.
    -   `nym-vpn-android/` Kotlin app + Rust via uniffi; `nym-vpn-apple/` Swift/SwiftUI apps with uniffi.
    -   `wireguard/`, `nym-vpn-windows/` OS glue (Wintun/WFP) and prebuilt libs in `build/`.

-   Cross-component interfaces:

    -   gRPC: `.proto` compiled by `nym-vpn-core/crates/nym-vpn-proto` (tonic/tonic-build). Use types from `nym_vpn_proto` in Rust. Edit proto there, then rebuild to regenerate.
    -   Mobile bindings: `nym-vpn-core` crates expose APIs via uniffi; Android/Apple consume generated Kotlin/Swift modules.
    -   Desktop types: Rust → TS types via `ts-rs`; run from `nym-vpn-app` when Rust types change.

-   Day-to-day workflows (examples):

    -   Core build (daemon + CLI): in `nym-vpn-core/` run a normal Cargo build (e.g., build the `nym-vpnd` and `nym-vpnc` packages). Respect workspace features and profiles.
    -   Desktop app: in `nym-vpn-app/` install deps, then:
        -   Generate TS types from Rust: run the TS type generation script.
        -   Dev (Tauri): start the desktop app in dev mode.
        -   Build (Tauri): produce a desktop bundle for your OS.
    -   Android/iOS/macOS: follow platform READMEs for NDK/Xcode setup, uniffi generation, and Gradle/Xcode builds. Don’t hand-edit generated uniffi code.

-   Conventions and gotchas:

    -   Rust: edition 2024; clippy config lives in `nym-vpn-core/`. Keep lints green; prefer small, composable crates. Public API changes require updating consumers (daemon, CLI, uniffi, TS types).
    -   Logging: use `tracing`; enable via `RUST_LOG` (e.g., `info,nym_vpnd=trace`).
    -   Important env flags used during dev/debug: `NYM_DISABLE_LOCAL_DNS_RESOLVER`, `NYM_DISABLE_OFFLINE_MONITOR`, `NYM_USE_PATH_MONITOR`, `NYM_FIREWALL_DEBUG` (see `nym-vpn-core/README.md`).
    -   Codegen output (tonic/uniffi/ts-rs) is generated — do not hand-edit. Change sources, then re-run generators/build.
    -   Windows: firewall/WFP and WireGuard/Wintun require admin context and correct prebuilt libs; see `nym-vpn-core/README.md` and `nym-vpn-windows/`.

-   Integration patterns to follow:

    -   New daemon API: update proto in `nym-vpn-proto`, rebuild to regenerate Rust stubs, implement server in `nym-vpnd` (e.g., `crates/nym-vpnd/src/service/`), and update clients (`nym-vpnc`, uniffi, desktop if applicable).
    -   New UI-visible types: add/adjust Rust structs with `ts-rs` derives in core crates, then regenerate TS in `nym-vpn-app`.
    -   Mobile API surface: expose via uniffi crates in `nym-vpn-core`, regenerate Kotlin/Swift bindings, and wire in Android/iOS projects.

-   Quality gates for PRs (keep changes tight and reproducible):

    -   Build `nym-vpn-core` workspace (at least `nym-vpnd` and `nym-vpnc`) for your target OS.
    -   Desktop: from `nym-vpn-app/`, generate TS types, run lint/format, and ensure dev build starts.
    -   Format/lint: `cargo fmt`/clippy in core; ESLint/Prettier in app. Don’t commit artifacts under `target/` or generated files unless they are intended outputs.

-   Where to look for details:

    -   Root `README.md` (architecture), plus platform docs: `nym-vpn-core/README.md`, `nym-vpn-app/README.md`, `nym-vpn-android/README.md`, `nym-vpn-apple/README.md`.

-   When in doubt: prefer expanding existing patterns (proto → tonic, uniffi, ts-rs) over inventing new IPC/FFI paths. Keep platform-specific logic in the platform crates/dirs and reuse shared crates from `nym-vpn-core`.
