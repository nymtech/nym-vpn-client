## 1. Frontend test scaffold

- [x] 1.1 Add dev dependencies: `vitest`, `@testing-library/react`, `@testing-library/jest-dom`, `@testing-library/user-event`, `jsdom` (pin versions compatible with Vite 8 / rolldown)
- [x] 1.2 Create `vitest.config.ts` (separate from `vite.config.ts`) with `environment: 'jsdom'`, globals enabled, and a setup file reference
- [x] 1.3 Create the test setup file (register `@testing-library/jest-dom` matchers; install `@tauri-apps/api/mocks` `mockIPC` and `clearMocks()` in `afterEach`)
- [x] 1.4 Add `"test": "vitest run"` script to `package.json` (and optionally `"test:watch": "vitest"`)
- [x] 1.5 Verify the empty suite runs: `npm run test` exits 0 with no tests

## 2. Shared frontend harness

- [x] 2.1 Implement `renderWithProviders` wiring i18next (test instance), the Zustand store, and react-router
- [x] 2.2 Add a `renderHook`-with-providers variant for the hook tier
- [x] 2.3 Add helpers to configure mocked Tauri IPC responses per test via `mockIPC`
- [x] 2.4 Write a smoke test that renders a trivial component through the harness to prove wiring

## 3. Tier 1 — pure logic tests

- [x] 3.1 Test `src/utils` (`index.ts`, `regex.ts`, `types.ts`) with edge-case inputs
- [x] 3.2 Test `src/errors` mapping/formatting helpers for each handled variant
- [x] 3.3 Test `src/state` helpers (`helper.ts` and other pure functions)

## 4. Tier 2 — Zustand store slices

- [x] 4.1 Test `createMainSlice` actions and derived state against a fresh store
- [x] 4.2 Test `createSocks5Slice` actions and derived state
- [x] 4.3 Test the gateways slice(s) under `store/slices/gateways`

## 5. Tier 3 — hooks (every hook in `src/hooks`)

- [x] 5.1 Pure/utility hooks: `useDebounce`, `useClickAway`, `useAnimatedNavigate`, `useScore`
- [x] 5.2 IPC/command hooks: `useConnect`, `useClipboard`, `useCustomDns`, `useLogout`, `useAutostart`, `useNodeListData`, `useRefreshAccountSummary`
- [x] 5.3 Event/watcher hooks: `useDeepLink`, `useGatewayIndependenceWatcher`, `useDesktopNotifications`, `useNotify`, `useToast`
- [x] 5.4 i18n hooks: `useI18nError`, `useI18nAccountState`, `useI18nProgressMsg`, `useI18nTunnelError`, `useLang`
- [x] 5.5 Confirm every `src/hooks/use*.ts` has a matching test file (no hook exempted)

## 6. Tier 4 — components (every component, tested equally)

- [x] 6.1 `ui/` inputs & controls: `Button`, `ButtonIcon`, `ButtonText`, `Switch`, `CardSwitch`, `RadioGroup`, `Slider`, `TextInput`, `TextArea`, `Link`, `ThemeSetter`
- [x] 6.2 `ui/` display & feedback: `Toast`, `Progress`, `Spinner`, `Skeleton`, `PulseDot`, `DaemonDot`, `InfoBanner`, `BetaPill`, `TopBar`, `FlagIcon`, `MsIcon`, `LewesIcon`, `SmileyIcon`
- [x] 6.3 `ui/` composite/interactive: `Dialog`, `ConfirmationDialog`, `DraggableList`, `CardNew`, `SettingsMenuCard`, `SettingsMenuCardBig`, `PageAnim`, `ScrambleIn`, `StaggeredText`
- [x] 6.4 `components/`: `BackNavigationConfirmationDialog`, `PrivyButton`, `ToastIcon`, `ToastList` (include a `user-event` interaction test)
- [x] 6.5 `screens/` — welcome, Onboarding, technical-opt-in, account
- [x] 6.6 `screens/` — home (all views)
- [x] 6.7 `screens/` — node (all views)
- [x] 6.8 `screens/` — settings (all views)
- [x] 6.9 Cover props variants, conditional rendering, and interactions (not just smoke renders) for components that branch
- [x] 6.10 Confirm every component module under `ui/`, `components/`, and `screens/` has a matching test file (completeness check: 0 missing)

## 7. Backend unit tests

- [x] 7.1 Add `#[cfg(test)]` tests for `country.rs` (code/region mapping)
- [x] 7.2 Add tests for `cli.rs` argument parsing
- [x] 7.3 Add tests for `fs/path` and `fs/util` helpers (fs/util: check_dir/check_file; fs/path is Lazy-static I/O over real user dirs — not unit-testable)
- [x] 7.4 Add tests for `db` serialization/round-trip logic (Key Display/iter contract; `Db` itself needs a real sled store — deferred)
- [x] 7.5 Add tests for `error.rs` variant mapping
- [x] 7.6 Add tests for `vpnd/deeplink` parsing and `vpnd/feature_flags` (`FeatureFlags::from` group-flag mapping: quic/domain_fronting/zknym, missing→false, non-group→absent, only literal "true")
- [x] 7.7 Add tests for `vpnd/config` — `MixnetTrafficConfig` ↔ `lib` round-trip; `MixnetTrafficDefaults::get()` well-formedness (transitively covers MixingDelay/BackgroundCoverTrafficRate/ContinuousTrafficSendingRate); and `VpndConfig::from_lib` via `lib::VpnServiceConfig::default()` + field overrides — asserts field mapping, node conversions, and the `enable_two_hop → vpn_mode` branch (Wg/Mixnet). The `ExitPoint::Address` `Err` path is left uncovered (constructing `Box<Recipient>` needs real base58 crypto keys — disproportionate for one passthrough `return Err`)
- [x] 7.8 Verify `cargo test` runs green in `src-tauri` and still regenerates `src/types/tauri.ts` (123 passed; `export_bindings_*` filter confirmed)

## 8. CI regression gating

- [x] 8.1 Un-stub the "Run tests" step in `.github/workflows/ci-nym-vpn-app-js.yml` to run `npm run test`
- [x] 8.2 Split `ci-nym-vpn-app-rust.yml`: keep a "Generate TS types" step (`cargo test export_bindings`) and add a separate "Run tests" step (`cargo test`); ts-rs `export_bindings_*` prefix confirmed against the test output
- [x] 8.3 Confirm the JS and Rust workflows run concurrently on a PR (separate workflows / runners) — no serialization between frontend and backend tests
- [ ] 8.4 Confirm both workflows fail their job when a test fails (verify via a temporary failing test on a branch, then revert) — requires pushing to CI; not verifiable locally

## 9. Documentation

- [x] 9.1 Update `CLAUDE.md` / `README.md` with how to run frontend (`npm run test`) and backend (`cargo test`) tests
