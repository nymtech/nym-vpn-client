## Why

`nym-vpn-app` ships with no automated test coverage on the frontend and only incidental
coverage on the backend (a handful of `#[cfg(test)]` modules under `fs/app_discovery`).
Regressions in state logic, IPC-facing types, or UI behavior can only be caught by manual
testing, and nothing prevents a broken change from merging. We want a test suite that runs
in CI and blocks PRs on failure, giving both halves of the app a regression safety net.

## What Changes

- **Frontend (`src/`)**: Introduce **Vitest** with a dedicated `vitest.config.ts` and a
  `jsdom` environment. Add unit tests across four tiers of testability:
  1. **Pure logic** — `src/utils`, `src/errors`, `src/state` helpers.
  2. **Zustand store slices** — `src/store/slices` (main, socks5, gateways).
  3. **Hooks** — `src/hooks/*` via `renderHook`, using `@tauri-apps/api/mocks`
     (`mockIPC`/`clearMocks`) and an i18next test instance.
  4. **Components** — `screens/`, `ui/`, `components/` via `@testing-library/react`.
- Add a shared **test harness** (`renderWithProviders` wiring i18n + Zustand store +
  react-router + Tauri mocks) as an explicit deliverable — tiers 3–4 depend on it.
- Add a `"test": "vitest run"` npm script and new devDependencies: `vitest`,
  `@testing-library/react`, `@testing-library/jest-dom`, `@testing-library/user-event`,
  `jsdom`.
- **Backend (`src-tauri/`)**: Add `#[cfg(test)]` unit tests using plain `cargo test`
  (matching the existing house style), with `#[tokio::test]` for async. Target pure logic:
  `country.rs`, `cli.rs` parsing, `fs/path`, `fs/util`, `db` serde, `error.rs` mapping,
  `vpnd/deeplink` parsing, `vpnd/feature_flags`, `vpnd/config`. No test-framework swap;
  Tauri mock-runtime command-handler tests are explicitly deferred to a later effort.
- **CI (primary goal — regression gating)**:
  - `ci-nym-vpn-app-js.yml`: un-stub the already-present commented "Run tests" step to run
    `npm run test`.
  - `ci-nym-vpn-app-rust.yml`: `cargo test` already runs (as ts-rs codegen), so new backend
    tests gate automatically; optionally split into a dedicated test step for clarity.
  - No coverage thresholds initially — regression gating only.

## Capabilities

### New Capabilities
- `testing`: Automated unit-test coverage for the nym-vpn-app frontend (Vitest/jsdom, all
  four testability tiers plus a shared provider harness) and backend (`cargo test`
  `#[cfg(test)]` modules), wired into the existing CI workflows so PRs are blocked on test
  failure.

### Modified Capabilities
<!-- No existing capability's requirements change; this adds a new orthogonal capability. -->

## Impact

- **New files**: `vitest.config.ts`, test setup file, `renderWithProviders` harness,
  `*.test.ts(x)` files across `src/`, `#[cfg(test)]` modules across `src-tauri/src/`.
- **Modified**: `package.json` (`test` script + devDependencies), `package-lock.json`.
- **CI**: `.github/workflows/ci-nym-vpn-app-js.yml` (un-stub test step);
  `.github/workflows/ci-nym-vpn-app-rust.yml` (optional step split).
- **Seam**: `cargo test` regenerates `src/types/tauri.ts` via ts-rs as a side effect; new
  `#[cfg(test)]` tests coexist with the generated `export_bindings_*` tests.
- **No production runtime code changes** — this is additive test infrastructure. Some pure
  functions may need minor extraction/`pub(crate)` visibility to be unit-testable.
