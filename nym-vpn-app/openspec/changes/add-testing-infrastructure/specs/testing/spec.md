## ADDED Requirements

### Requirement: Frontend test runner

The frontend SHALL provide a Vitest-based unit-test runner configured in a dedicated
`vitest.config.ts` (separate from `vite.config.ts`) using the `jsdom` test environment. A
`test` npm script SHALL run the suite non-interactively via `vitest run`.

#### Scenario: Test script runs the suite headlessly

- **WHEN** `npm run test` is executed in `nym-vpn-app`
- **THEN** Vitest runs all `*.test.ts`/`*.test.tsx` files once and exits with a non-zero
  status if any test fails

#### Scenario: jsdom environment is available to tests

- **WHEN** a test renders a React component or touches DOM globals (`window`, `document`)
- **THEN** the jsdom environment provides them without additional per-test setup

#### Scenario: Test config is isolated from the build config

- **WHEN** the production build runs (`npm run build`)
- **THEN** it uses `vite.config.ts` and is unaffected by test-only configuration in
  `vitest.config.ts`

### Requirement: Frontend pure-logic coverage

The frontend SHALL have unit tests for pure logic modules in `src/utils`, `src/errors`, and
`src/state` helpers that require no DOM, store, or IPC mocking.

#### Scenario: Utility functions are covered

- **WHEN** a function in `src/utils` (e.g. `index.ts`, `regex.ts`) is tested with
  representative inputs including edge cases
- **THEN** the test asserts its return value directly with no mocks

#### Scenario: Error mapping is covered

- **WHEN** an error-handling helper in `src/errors` maps or formats an error value
- **THEN** a test asserts the mapped output for each handled variant

### Requirement: Frontend store-slice coverage

The frontend SHALL have unit tests for the Zustand store slices in `src/store/slices`
(main, socks5, gateways) exercising their actions and derived state.

#### Scenario: Slice action updates state

- **WHEN** a slice action is dispatched against a freshly instantiated store
- **THEN** the test asserts the resulting state matches the expected value

#### Scenario: Slices are tested in isolation

- **WHEN** a single slice is under test
- **THEN** it is instantiated without requiring a running Tauri backend or DOM

### Requirement: Shared frontend test harness

The frontend SHALL provide a shared `renderWithProviders` test harness that wires the
i18next instance, the Zustand store, react-router, and Tauri API mocks so that hook and
component tests share one consistent setup.

#### Scenario: Rendering a component under providers

- **WHEN** a test renders a component via `renderWithProviders`
- **THEN** the component has access to i18n translation, store state, and routing without
  each test re-wiring providers

#### Scenario: Tauri IPC is mocked by default

- **WHEN** code under test calls a Tauri command via `@tauri-apps/api`
- **THEN** the harness intercepts it with `mockIPC` and mocks are cleared between tests via
  `clearMocks`

### Requirement: Frontend hook coverage

The frontend SHALL have unit tests for **every** hook in `src/hooks` using `renderHook`, the
shared harness, mocked Tauri IPC, and an i18next test instance. Coverage is uniform: no hook
is exempted as "low value" — each hook has at least one test file exercising its primary
behavior, and hooks with branching, IPC, i18n, or event-listener logic are tested across
their meaningful states.

#### Scenario: Every hook has a test file

- **WHEN** the hook tier is complete
- **THEN** each `src/hooks/use*.ts` module has a corresponding test that renders it via
  `renderHook`

#### Scenario: Hook logic is exercised

- **WHEN** a hook (e.g. `useDebounce`) is rendered via `renderHook` with controlled inputs
- **THEN** the test asserts its returned value and behavior over state changes, including
  edge/error branches where present

#### Scenario: Hook consuming IPC uses mocks

- **WHEN** a hook invokes a Tauri command
- **THEN** the mocked IPC returns a controlled response and the test asserts the hook's
  reaction, including both success and failure responses

### Requirement: Frontend component coverage

The frontend SHALL have unit tests for React components across `ui/`, `components/`, and
`screens/` using `@testing-library/react` through the shared harness. Coverage is uniform
and thorough: **every** component is tested equally rather than a prioritized subset — each
component has at least a render test, and components with props variants, conditional
rendering, or user interaction are tested across those states.

#### Scenario: Every component has a test file

- **WHEN** the component tier is complete
- **THEN** each component module under `ui/`, `components/`, and `screens/` has a
  corresponding test that renders it via `renderWithProviders`

#### Scenario: Component renders expected content

- **WHEN** a component is rendered via `renderWithProviders`
- **THEN** the test queries the DOM and asserts the presence of expected elements/text

#### Scenario: Props variants and conditional rendering are covered

- **WHEN** a component renders differently based on props or state
- **THEN** tests assert each meaningful variant/branch

#### Scenario: User interaction triggers expected behavior

- **WHEN** a simulated user interaction (click, input) occurs via `user-event`
- **THEN** the test asserts the resulting UI change or mocked command invocation

### Requirement: Backend unit-test coverage

The backend SHALL have `#[cfg(test)]` unit tests run via `cargo test` covering pure logic in
`country.rs`, `cli.rs` argument parsing, `fs/path`, `fs/util`, `db` serialization,
`error.rs` mapping, `vpnd/deeplink` parsing, `vpnd/feature_flags`, and `vpnd/config`. Async
code SHALL be tested with `#[tokio::test]`.

#### Scenario: Pure-logic function is covered

- **WHEN** a targeted pure function is tested with representative and edge-case inputs
- **THEN** the test asserts its output using standard `assert!`/`assert_eq!`

#### Scenario: Async logic uses the tokio test macro

- **WHEN** an async function is under test
- **THEN** it is exercised inside a `#[tokio::test]` and awaited

#### Scenario: Backend tests coexist with ts-rs codegen

- **WHEN** `cargo test` runs in `src-tauri`
- **THEN** the new `#[cfg(test)]` tests run alongside the generated `export_bindings_*`
  tests and neither disrupts the other

### Requirement: CI regression gating

Test execution SHALL run in the existing CI workflows on pull requests touching
`nym-vpn-app`, and a test failure SHALL fail the corresponding CI job so the PR cannot merge.

#### Scenario: Frontend tests gate the JS workflow

- **WHEN** a pull request modifies `nym-vpn-app` and `ci-nym-vpn-app-js.yml` runs
- **THEN** the workflow runs `npm run test` and fails the job if any frontend test fails

#### Scenario: Backend tests gate the Rust workflow

- **WHEN** a pull request modifies `nym-vpn-app/src-tauri` and `ci-nym-vpn-app-rust.yml` runs
- **THEN** the workflow runs the backend test suite and fails the job if any backend test
  fails

#### Scenario: No coverage threshold blocks merges

- **WHEN** the test suites pass but code coverage is below any particular percentage
- **THEN** the CI jobs still succeed, because gating is on test failure only, not coverage

### Requirement: Separate Rust type-generation and test steps

The Rust CI workflow SHALL run ts-rs type generation and unit-test execution as two
distinct, clearly-named steps, so a unit-test failure is not misattributed to type
generation.

#### Scenario: Type generation and tests are distinct steps

- **WHEN** `ci-nym-vpn-app-rust.yml` runs
- **THEN** there is a dedicated step that regenerates `src/types/tauri.ts` and a separate
  dedicated step labeled for running the unit-test suite

#### Scenario: A failing unit test is attributed correctly

- **WHEN** a backend unit test fails in CI
- **THEN** the failure surfaces on the "run tests" step, not on the type-generation step

### Requirement: Frontend and backend tests run in parallel

Frontend and backend test suites SHALL execute concurrently on pull requests, rather than
serialized behind one another.

#### Scenario: JS and Rust workflows run concurrently

- **WHEN** a pull request modifies files that trigger both `ci-nym-vpn-app-js.yml` and
  `ci-nym-vpn-app-rust.yml`
- **THEN** the two workflows run on separate runners at the same time, so frontend and
  backend tests are not gated on one another's completion
