# Tauri anonymous account creation + recovery-phrase reveal (Linux)

Spec date: 2026-05-21
Branch: `feature/tauri_anon_account`
Reference: Android implementation on `android/fdroid-anonymous-account` (commit `36e990bc1`)

## Goal

Port the Android F-Droid anonymous-account flow to the Tauri desktop app on **Linux only**. Windows behaviour must not change.

Two user-visible features:

1. **Local anonymous account creation.** The "Sign up anonymously" button on the welcome screen generates a 24-word BIP39 mnemonic in the daemon's local store, with no backend registration. Registration with the nym-vpn-api is deferred until the user clicks "Get a plan" (web checkout) — at that point the daemon POSTs to `/api/public/v1/account` so the website can sign the user in.
2. **Recovery-phrase reveal page in settings.** A new screen under `/settings/account/recovery-phrase` that shows the stored mnemonic after a per-call polkit prompt. Available for any account mode (locally-generated, imported, Privy).

Plus a persistent home-screen banner that nags the user to back up their recovery phrase, but only after the anonymous account has an active subscription.

## Scope

In scope:

- Daemon (`nym-vpnd`): 4 new gRPC RPCs, polkit policy file, account-storage schema extension.
- Account-controller (`nym-vpn-account-controller`): new commands + handlers, idempotent registration.
- Tauri Rust backend: new `#[cfg(target_os = "linux")]` commands + vpnd wrappers.
- Tauri frontend (React): welcome-flow rewire, home banner, settings row, reveal page.

Out of scope (explicit non-goals):

- Windows or macOS support for any of the above. Daemon RPCs are added unconditionally so the proto stays uniform, but `GetStoredMnemonic` returns `Unimplemented` off-Linux and the Tauri client never calls the new RPCs on non-Linux builds.
- True memory zeroization of the mnemonic string in the JS heap (impossible without rewriting React state plumbing). Mitigation: component-local state, drop on unmount.
- Encrypting the stored mnemonic at the daemon level with the polkit-confirmed user password. Polkit only gates *access to* the reveal call.
- Re-auth on `confirm_mnemonic_backup` or any other RPC. Per-call polkit is `GetStoredMnemonic` only.

## Architecture

### Daemon (Rust core)

**Account storage schema extension** in `nym-vpn-store`:

Add three booleans to the persisted account record:

| Field | Set by | Used for |
|---|---|---|
| `is_locally_generated: bool` | `CreateAccount` → `true`; `StoreAccount` → `false` | Distinguishes anonymous-generated from imported/Privy accounts. Drives banner + checkbox visibility. |
| `is_registered_with_api: bool` | `RegisterAnonymousAccount` (success), or implicit during a successful first sync | Idempotency of registration; banner triggering. |
| `is_backup_confirmed: bool` | `ConfirmMnemonicBackup` | Banner visibility. |

All three default to `false`. The fields must survive daemon restart (use the same persisted record as the mnemonic itself).

**Account-controller** (`nym-vpn-account-controller/src/commands/`):

- `dispatch.rs`: extend `AccountCommand` with two new variants — `GetStoredMnemonic(ReturnSender<String, AccountCommandError>)` and `ConfirmMnemonicBackup(ReturnSender<(), AccountCommandError>)`. Both also need to be handled in `handle_command_error()`.
- `handler.rs`:
  - `handle_create_account`: existing — additionally sets `is_locally_generated = true` in the new field on the freshly created record.
  - `handle_store_account`: existing — explicitly sets `is_locally_generated = false`. The Privy / web-deeplink import path (`handle_deeplink_store_account` in `vpn_service.rs`) also routes through `store_account`; verify the flag is set to `false` on that path too.
  - `handle_register_anonymous_account`: existing (from Android branch) — make idempotent. Read `is_registered_with_api` from the stored account; if `true`, return `Ok(RegisterAccountResponse { account_token: String::new() })` without HTTP. Otherwise call `register_anonymous_account()` on the API client; on success, set the flag.
  - `handle_get_stored_mnemonic` (new): read the recovery phrase out of `nym-vpn-store` and return the raw string. Error if no account stored (`AccountCommandError::NoAccountStored`).
  - `handle_confirm_mnemonic_backup` (new): flip the flag in storage; idempotent on repeat.
- `command_sender.rs`: add `get_stored_mnemonic()` and `confirm_mnemonic_backup()` wrappers analogous to existing patterns. Already has `register_anonymous_account` and `create_account_command`.

**vpn-service** (`nym-vpn-lib/src/service/vpn_service.rs`):

- Remove the existing `#[cfg(any(target_os = "android", target_os = "ios"))]` gate on `VpnServiceCommand::RegisterAnonymousAccount` and `handle_register_anonymous_account`. Add `target_os = "linux"` to the allowed set (so we don't expose it on Windows/macOS where there's no UI for it). Concretely: `#[cfg(any(target_os = "android", target_os = "ios", target_os = "linux"))]`.
- Add `VpnServiceCommand::CreateAccount` to the same cfg-set if it isn't already cross-platform (it is per the current code, but re-check during implementation).
- Add `VpnServiceCommand::GetStoredMnemonic(oneshot::Sender<Result<String, AccountCommandError>>, ())` — Linux only.
- Add `VpnServiceCommand::ConfirmMnemonicBackup(oneshot::Sender<Result<(), AccountCommandError>>, ())` — Linux only.

### Proto (`nym-vpn-core/crates/nym-vpn-proto/proto/nym_vpn_service.proto`)

Add four RPCs to `service NymVpnService` (unconditional):

```proto
// Generate a new mnemonic and store it locally. Does NOT register with the backend.
rpc CreateAccount(google.protobuf.Empty) returns (AccountCommandResponse) {}

// Register the locally-stored account with the nym-vpn-api. Idempotent — no-op if
// the account is already registered.
rpc RegisterAnonymousAccount(google.protobuf.Empty) returns (AccountCommandResponse) {}

// Reveal the stored recovery phrase. On Linux the daemon performs a per-call polkit
// authentication check; on other platforms returns Unimplemented.
rpc GetStoredMnemonic(google.protobuf.Empty) returns (GetStoredMnemonicResponse) {}

// Mark the recovery phrase as "saved by user" — clears the home-screen backup banner.
rpc ConfirmMnemonicBackup(google.protobuf.Empty) returns (google.protobuf.Empty) {}

message GetStoredMnemonicResponse {
  string mnemonic = 1;
}
```

Extend the existing `AccountControllerState` (or whichever message backs `GetAccountState`) with the three new booleans `is_locally_generated`, `is_registered_with_api`, `is_backup_confirmed`. Confirm the field positions during implementation.

### Polkit policy (Linux only)

New polkit action installed at runtime by the daemon, alongside the existing `com.nymvpn.vpnd.unix-access` action. The existing `unix-access` action is written on first use by the auth code in `nym-ipc/src/authentication/linux.rs` (lines ~65-78): it calls `proxy.enumerate_actions("")`, and if the action id isn't found, writes `/usr/share/polkit-1/actions/{ACTION_ID}.policy` from a `POLKIT_POLICY` const. The new `reveal-mnemonic` action follows the same pattern — add a sibling const + the same auto-install logic in the daemon's `GetStoredMnemonic` handler. No packaging changes needed.

```xml
<action id="com.nymvpn.vpnd.reveal-mnemonic">
  <description>Reveal stored recovery phrase</description>
  <message>Authentication is required to reveal the recovery phrase</message>
  <defaults>
    <allow_any>auth_admin</allow_any>
    <allow_inactive>auth_admin</allow_inactive>
    <allow_active>auth_self</allow_active>
  </defaults>
</action>
```

The daemon's `GetStoredMnemonic` gRPC handler (Linux build, in `nym-vpnd/src/command_interface.rs`) extracts the request's peer credentials and calls `AuthorityProxy::check_authorization` with action id `com.nymvpn.vpnd.reveal-mnemonic` and the `AllowUserInteraction` flag. Same 60-second user-interaction timeout pattern as `nym-ipc/src/authentication/linux.rs`. On authorization denied / cancelled / timed out → return `tonic::Status::permission_denied`.

On Windows / macOS the handler is `#[cfg(not(target_os = "linux"))]` and returns `tonic::Status::unimplemented`.

This re-runs the polkit prompt on **every** call to `GetStoredMnemonic` — there is no session caching. Other RPCs continue to use the existing connect-time auth only.

### ts-rs / type generation

The new fields on `AccountControllerState` will reach `src/types/tauri.ts` through the existing ts-rs derive in `nym-vpn-lib-types`. Run `npm run tsgen` after the Rust changes are in.

### Tauri Rust backend (`nym-vpn-app/src-tauri/`)

New commands in `src/commands/account.rs`, all `#[cfg(target_os = "linux")]`:

| Tauri command | Calls vpnd | Returns |
|---|---|---|
| `create_local_account` | `vpnd.create_account()` | `()` |
| `register_anonymous_account` | `vpnd.register_anonymous_account()` | `()` |
| `get_stored_mnemonic` | `vpnd.get_stored_mnemonic()` | `String` |
| `confirm_mnemonic_backup` | `vpnd.confirm_mnemonic_backup()` | `()` |

`create_local_account` reuses the same tunnel-state guard as the existing `add_account` (must be `TunnelState::Disconnected`).

New thin wrappers in `src/vpnd/account.rs` mirroring the existing `store_account` / `forget_account` patterns. Error mapping through `handle_rpc_error`.

New variants in `src/error.rs` `ErrorKey`:

- `MnemonicRevealDenied` — polkit refused / cancelled / timed out (maps to `tonic::Status::permission_denied`).
- `MnemonicNotAvailable` — daemon returned `NoAccountStored` (defensive — UI shouldn't allow it).

Registration in `src/main.rs` `invoke_handler!` inside `#[cfg(target_os = "linux")]`.

### Frontend (React)

**Platform gate.** Add a `useIsLinux()` hook that reads the existing platform indicator (look up during implementation — likely in the main Zustand slice or via a Tauri command; add one if missing). Every new piece of UI below is gated by this hook.

**Welcome flow** (`src/screens/welcome/components/Signup.tsx`):

The existing `signup-anonymous-button` callback `handleCreateAccount` branches on `isLinux`:

- **Linux**: `await invoke('create_local_account')` → `dispatch({ type: 'set-account', stored: true })` → `await CCache.del('cache-account-id')` + `CCache.del('cache-device-id')` → `dispatch({ type: 'reset-error' })` → navigate to home (or technical-optin if not yet seen). No browser open, no deeplink listener, no `store_deeplink_account`.
- **Non-Linux**: existing web-based flow unchanged.

On error: surface a toast, stay on the Signup screen, do not navigate.

**Home — backup banner**:

New component `MnemonicBackupBanner.tsx` rendered above the connect button. Visibility:

```
isLinux
  && accountState === 'Ready'
  && account.is_locally_generated
  && !account.is_backup_confirmed
```

Contents: warning icon + copy (i18n key `home.backup-banner.title` and `home.backup-banner.description`) + button-link "Reveal recovery phrase" → navigates to `routes.revealMnemonic`. The banner is non-dismissible; the only way to clear it is to tick the checkbox on the reveal page.

**Home — connect button** (`src/screens/home/components/ConnectionButton.tsx` or equivalent):

When the user has a stored account but no active subscription, render "Get a plan" instead of "Connect", mirroring the Android branch. Trigger condition (high-level): `accountStored && !canConnect`, where `canConnect` is the existing set of account states from which `connect_tunnel` works (e.g. `Ready`). The exact set is implementation-dependent — pin during implementation by looking at the current Home `ConnectionButton` enable condition. For never-registered local accounts the daemon will likely report a sync-error or LoggedOut-with-stored-account state; the new flags `is_locally_generated && !is_registered_with_api` are a sufficient secondary signal but the primary trigger is "stored account + not connectable".

`onClick` handler:

```
setLoading(true)
try {
  await invoke('register_anonymous_account')  // idempotent in daemon — safe to call for imported/Privy accounts too
  const url = await invoke('get_autologin_deeplink', { locale, kind: 'CreateAccount' })
  openUrl(url.url)
} catch (e) {
  toast.error(t('home.get-plan.error'))
} finally {
  setLoading(false)
}
```

Because the daemon's `RegisterAnonymousAccount` is idempotent, this handler is safe regardless of how the account was originally created (anonymous-never-registered, anonymous-already-registered, imported, Privy). For already-registered accounts it's a single fast no-op round-trip before opening the URL.

Behaviour on non-Linux: unchanged (no new "Get a plan" rendering; existing web signup remains entry point).

**Settings → Account** (`src/screens/settings/account/Account.tsx`):

New `AccountSettingRow` "Recovery phrase" with chevron → navigates to `routes.revealMnemonic`. Visibility: `isLinux && accountStored`. Shown for all account modes (locally-generated, imported, Privy). Hidden on Windows.

**Reveal page** (new file `src/screens/settings/account/RevealMnemonic.tsx`, route `revealMnemonic: '/settings/account/recovery-phrase'`):

State machine (all component-local `useState`, never Zustand):

- `idle`: heading, warning callout, "Reveal" button. Back/close button in topbar → returns to `routes.accountSettings`, no state change.
- `prompting`: shown while `get_stored_mnemonic` is in flight (polkit dialog is open on the user's desktop, system-modal — not in our app). Spinner inside the page.
- `revealed`: 24-word grid + copy-to-clipboard button. Conditionally renders below the grid:
  - If `account.is_locally_generated && !account.is_backup_confirmed`: a checkbox "I have saved my recovery phrase in a safe place" + Continue button (disabled until checked). On Continue → `await invoke('confirm_mnemonic_backup')` → `await invoke('refresh_account_state')` (so banner clears within ~1 s) → navigate back to Settings → Account.
  - Otherwise (imported / Privy / already-confirmed): no checkbox row. Only a Back button.

Navigation rules:

- Back / close at any time → returns to Settings → Account with no state change. If the user had not confirmed, the banner persists. Mnemonic is dropped from memory.
- On unmount (`useEffect` cleanup): set the mnemonic state to `undefined`. JS strings are immutable so we can't zero-overwrite; this is acceptable.
- Every fresh visit to the page starts in `idle` — polkit re-prompts on the next Reveal.

On `MnemonicRevealDenied` error: toast "Authentication was cancelled or denied.", state returns to `idle`. User can press Reveal again.

**Router** (`src/router.tsx` + `src/types/routes.ts`):

Add `revealMnemonic: '/settings/account/recovery-phrase'` nested under `accountSettings`. Lazy-loaded import in `src/screens/index.ts`.

**Zustand store** (`src/store/slices/createMainSlice.ts`):

When `accountState` / `AccountControllerState` is fetched, also store the three new flags. Add selectors:

- `useAccountLocallyGenerated()`
- `useAccountRegistered()`
- `useAccountBackupConfirmed()`

### i18n

New translation keys (English source under `src/i18n/en/`, then crowdin for the other locales):

| Namespace | Key | Approx. copy |
|---|---|---|
| `login` | `signup.signup-anonymous-button` | (existing; may need re-copy now that it doesn't open browser) |
| `home` | `backup-banner.title` | "Save your recovery phrase" |
| `home` | `backup-banner.description` | "It's the only way to recover your account. Save it now." |
| `home` | `backup-banner.action` | "Reveal" |
| `home` | `get-plan.button` | "Get a plan" |
| `home` | `get-plan.error` | "Could not start checkout. Try again." |
| `settings` | `account.recovery-phrase.row` | "Recovery phrase" |
| `recovery-phrase` (new namespace) | `title` | "Recovery phrase" |
| `recovery-phrase` | `warning` | "Anyone with this phrase can access your account. Keep it private." |
| `recovery-phrase` | `reveal-button` | "Reveal" |
| `recovery-phrase` | `auth-denied-toast` | "Authentication was cancelled or denied." |
| `recovery-phrase` | `copy-button` | "Copy" |
| `recovery-phrase` | `copied-toast` | "Copied to clipboard" |
| `recovery-phrase` | `saved-checkbox` | "I have saved my recovery phrase in a safe place" |
| `recovery-phrase` | `continue-button` | "Continue" |

## Error handling

| Failure | Detection | UX |
|---|---|---|
| Polkit refused / cancelled / 60 s timeout | `tonic::Status::permission_denied` → `MnemonicRevealDenied` | Toast, return to `idle` on reveal page |
| Daemon unreachable during reveal | Existing `VpndError::FailedToConnectIpc` / `AuthenticationRequired` | Existing `SystemAuthentication.tsx` dialog handles re-auth; reveal page falls back to `idle` with generic error toast |
| `create_local_account` while account already stored | Daemon storage rejects double-store | Toast on Signup screen, do not navigate |
| `create_local_account` while tunnel ≠ Disconnected | Existing guard in `add_account` returns `BackendError::internal` | Toast on Signup screen, do not navigate |
| `register_anonymous_account` network failure | HTTP error from `nym-vpn-api-client` | Toast "Could not reach Nym account service. Try again.", do NOT open the URL |
| `register_anonymous_account` "no account stored" | `AccountCommandError::NoAccountStored` | Toast generic error, log, do NOT open the URL (defensive — should not happen) |
| Already registered (`is_registered_with_api == true`) | Daemon short-circuits | App proceeds to autologin URL — no error path |
| Backup checkbox raced with logout | `confirm_mnemonic_backup` returns "no account stored" | Treat as success in UI — banner is moot anyway |
| Daemon storage wiped externally | Next state refresh shows no account → flags reset | Welcome screen reachable; no stale banner |

## Privy / imported account on the reveal page

The reveal page is available for any account mode on Linux. What `GetStoredMnemonic` actually returns for Privy accounts (24-word BIP39 vs derivation key vs something else) is **TBD during implementation** — needs to be verified against `nym-vpn-store`'s recovery-phrase storage slot for the `Privy` mode. The display widget needs to know whether to render a 24-word grid or a single opaque string. If the format differs, branch on `account.mode` to pick the right renderer.

This is the only point in the spec marked TBD — it will be resolved by reading the store code during implementation, with a follow-up question to the user if the answer is ambiguous.

## Testing

### Unit tests

Rust core (account-controller):

- `handle_register_anonymous_account`: idempotency (pre-set flag → no HTTP, returns Ok) and happy path (flag flips after POST).
- `handle_create_account`: post-condition `is_locally_generated = true`, `is_registered_with_api = false`, `is_backup_confirmed = false`.
- `handle_store_account`: post-condition `is_locally_generated = false`.
- `handle_confirm_mnemonic_backup`: flag flips; idempotent on repeat.
- `handle_get_stored_mnemonic`: returns expected string; `AccountCommandError::NoAccountStored` when no account.

Polkit integration: extend the existing MockProxy / MockPrompter scaffolding in `nym-ipc/src/authentication/linux.rs` for the new `reveal-mnemonic` action. Cover authorized, denied, timeout, cancellation.

Daemon gRPC layer: integration test with a mocked `VpnService` — call each new RPC, assert correct mapping to `AccountCommand`. Assert `tonic::Status::permission_denied` on polkit-denied and `tonic::Status::unimplemented` on non-Linux build via a cfg-test.

Tauri Rust: existing tests are sparse. Confirm via `cargo test` that ts-rs regenerates `src/types/tauri.ts` with the three new fields.

### Frontend

This project has no JS test runner configured (verify during implementation). Manual QA only.

### Manual QA checklist (Linux)

1. Fresh install → Welcome → Sign up anonymously → lands on Home in LoggedOut+stored state. No browser opens.
2. Home shows "Get a plan" button. Click → registration succeeds (single HTTP POST to `/api/public/v1/account`) → browser opens to autologin URL. Banner not yet visible.
3. Simulate "subscription activated" → AccountState transitions to Ready → banner appears above Connect button.
4. Settings → Account → Recovery phrase row visible. Click → reveal page → Reveal → polkit prompt → enter password → 24 words display + copy button + checkbox.
5. Tick checkbox → Continue → back to Settings; banner gone on Home within ~1 s.
6. Re-open Reveal page → polkit prompt appears again (mnemonic was dropped). 24 words display, no checkbox (already confirmed).
7. Press back without ticking checkbox (before confirming) → banner persists; re-enter prompts polkit again.
8. Polkit cancel → toast appears, stay on reveal page idle, no navigation.
9. Logout (Forget Account) → reveal row hidden; welcome screen reachable.
10. Import existing mnemonic via PassphraseEnter → Reveal row visible, no checkbox/banner ever shown.
11. Privy social login → Reveal row visible; revealed content rendered per resolved format (see "Privy" section above).
12. Run on Windows build → no behavioural change (no banner, no reveal row, signup-anonymous still opens browser).

## Delivery

Single PR is acceptable since the changes are tightly coupled. Natural slicing if the PR grows too large:

- PR1: Rust core + proto + ts-rs types (daemon + Tauri backend commands, no UI).
- PR2: Frontend UI (welcome change, banner, reveal page, settings row).
- PR3 (follow-up, separate work): Windows port of OS auth + remove platform gates.
