## Context

`nym-vpn-app` is a Tauri 2 desktop client: a Rust backend (`src-tauri/`) driving an external
`nym-vpnd` daemon over gRPC, and a React 19 / TypeScript / Zustand frontend (`src/`). Today
there is no frontend test infrastructure at all, and backend coverage is limited to a few
`#[cfg(test)]` modules under `fs/app_discovery`. Two CI workflows already exist and are
path-filtered to the app:

- `ci-nym-vpn-app-js.yml` — runs tscheck, lint, fmt:check, build. It already contains a
  **commented-out** "Run tests" step invoking `npm run test`.
- `ci-nym-vpn-app-rust.yml` — runs fmt, clippy, then `cargo test` (labeled "Generate TS
  types" because ts-rs exports `src/types/tauri.ts` as a side effect of the test run), then
  tscheck.

This means the regression-gating hooks are largely pre-wired; the work is to populate tests
and activate the JS test step.

## Goals / Non-Goals

**Goals:**
- Vitest-based frontend unit tests in a separate `vitest.config.ts` with a `jsdom`
  environment, covering all four testability tiers (pure logic, store slices, hooks,
  components) plus a shared `renderWithProviders` harness.
- Backend `#[cfg(test)]` unit tests via plain `cargo test`, targeting pure logic.
- CI configured so a test failure blocks the PR on both the JS and Rust workflows.

**Non-Goals:**
- Code-coverage measurement or thresholds (gating is on failure only).
- Tauri mock-runtime tests of `#[tauri::command]` handlers (`tauri::test::mock_builder` /
  `MockRuntime`) — deferred to a later integration-test effort.
- End-to-end / integration tests spanning the app and a live `nym-vpnd`.
- Swapping the Rust test framework or adding heavyweight test dependencies.

## Decisions

### Frontend runner: Vitest with a separate config
Vitest integrates with the existing Vite (rolldown) toolchain and shares transforms, so no
separate Babel/Jest pipeline is needed. A dedicated `vitest.config.ts` keeps test concerns
(jsdom, setup files, globals) out of the tauri-tailored `vite.config.ts` (fixed port,
rolldown output groups). *Alternative considered:* a `test` block inside `vite.config.ts` —
rejected to avoid coupling the build path to test config.

### DOM environment: jsdom
`jsdom` is chosen over `happy-dom` for broader API completeness, which matters for the
component tier (react-router, focus/interaction via `user-event`). *Trade-off:* jsdom is
slower than happy-dom, accepted for fidelity at this stage.

### Shared harness as a first-class deliverable
Tiers 3–4 depend on i18next, the Zustand store, react-router, and Tauri IPC. A single
`renderWithProviders` (plus a `renderHook` variant) wiring these — and a global setup that
installs `@tauri-apps/api/mocks` `mockIPC` and `clearMocks` between tests — prevents each
test from re-inventing provider wiring. This is built before/with the first hook test.
*Alternative considered:* per-test provider wiring — rejected as brittle and duplicative.

### Backend: plain `cargo test` + `#[cfg(test)]`
Matches the existing house style (`fs/app_discovery`), adds zero dependencies, and
`#[tokio::test]` is already available (tokio `macros` feature enabled). Scope is pure logic
reachable without a Tauri runtime. *Alternatives considered:* `rstest`/`insta` for
table/snapshot tests (deferred — introduce only if ergonomics demand it); `tauri::test`
mock runtime for command handlers (deferred — integration-flavored, brittle).

### CI wiring: activate what exists
- JS: un-stub the existing commented "Run tests" step to run `npm run test`; add the `test`
  script and dev dependencies.
- Rust: `cargo test` already runs, so new `#[cfg(test)]` tests gate automatically. **Split**
  the single step into two clearly-named steps: a "Generate TS types" step that regenerates
  `src/types/tauri.ts` (e.g. `cargo test export_bindings`, filtering to the ts-rs generated
  tests), and a "Run tests" step that runs the full suite (`cargo test`). The generated
  export tests are trivial, so the small overlap in what runs is negligible; the payoff is
  correct failure attribution (a logic-test failure lands on "Run tests", not on codegen).
  Order: generate types first so the subsequent `tscheck` sees a current `tauri.ts`.
*Alternative considered:* a new standalone test workflow — rejected; the existing
path-filtered workflows are the right home.

### FE and Rust tests run in parallel — already true by construction
`ci-nym-vpn-app-js.yml` and `ci-nym-vpn-app-rust.yml` are independent workflows, both
triggered by the same `pull_request` event and running on separate runners. GitHub Actions
therefore executes them **concurrently** — frontend and backend tests already run in
parallel with no extra work. Each also uses a `fail-fast: false` OS matrix, so per-OS jobs
run in parallel too. We preserve this rather than merging the suites into one serialized
workflow. *Alternative considered:* a single combined workflow with parallel jobs — rejected;
it would collapse the existing clean separation for no gain.

### No coverage thresholds initially
The stated goal is regression gating (a failing test blocks merge), which is independent of
coverage percentage. Coverage tooling (`vitest --coverage`, `cargo-llvm-cov`) can be layered
on later without reworking this.

## Risks / Trade-offs

- **Tier 3–4 flakiness from async IPC/event mocks** → Centralize mocking in the harness with
  strict `clearMocks` between tests; keep component tests behavior-focused, not
  implementation-snapshot-focused.
- **Volume: uniform coverage of ~21 hooks + ~130 components/screens is large** → Accepted
  deliberately (thorough, equal coverage was a requirement). Mitigate cost with the shared
  harness and reusable per-directory patterns; the effort is breadth, not per-test
  complexity. Track completion by "every module has a test file" (see spec scenarios) rather
  than by a coverage percentage.
- **ts-rs regenerates `src/types/tauri.ts` during `cargo test`** → Existing behavior; new
  `#[cfg(test)]` modules only add tests to the run. If CI later needs to verify the committed
  file is current, that is a separate follow-up, not part of this change.
- **Rolldown-Vite / Vitest version compatibility** → Pin versions during implementation and
  verify `npm run test` runs green locally before wiring CI.
- **Some pure functions are private inside command modules** → May require minor
  `pub(crate)` visibility or extraction into a testable helper; keep such refactors minimal
  and behavior-preserving.
- **CI runtime increase** → Frontend tests are fast; backend `cargo test` already runs, so
  the marginal cost is small.

## Migration Plan

Additive only — no production runtime code changes and no rollback concerns. Sequence:
scaffold Vitest config + harness → add tests tier by tier (1→4) and backend modules →
activate the JS CI test step. Reverting is dropping the added files/steps.

## Open Questions

- Exact `cargo test` filter for the isolated "Generate TS types" step — confirm the ts-rs
  generated test-name prefix (expected `export_bindings_*`) during implementation.
- None on scope: the Rust CI step is split (decided), and all hooks/components are covered
  uniformly (no prioritization).
