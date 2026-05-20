# Tauri anonymous account (Linux) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Linux-only local anonymous account creation + deferred backend registration + per-call polkit-gated recovery-phrase reveal in settings to the Tauri desktop app. Windows behaviour must not change.

**Architecture:** Daemon owns the three new persisted flags (`is_locally_generated`, `is_registered_with_api`, `is_backup_confirmed`) on the account record. Four new gRPC RPCs (`CreateAccount`, `RegisterAnonymousAccount`, `GetStoredMnemonic`, `ConfirmMnemonicBackup`). `GetStoredMnemonic` is the only RPC gated by per-call polkit (Linux); on non-Linux it returns `Unimplemented`. Frontend rewires the existing "Sign up anonymously" button to call the local-create path, adds a home banner above the Connect button, adds a recovery-phrase row in Settings → Account, and adds a new Reveal page. All UI is gated behind a Linux-only platform check.

**Tech Stack:** Rust (account-controller, vpn-service, daemon, tonic gRPC, polkit via zbus_polkit, ts-rs), TypeScript/React 19 (Tauri 2, Zustand, React Router 7, i18next, Tailwind v4).

**Spec:** `docs/superpowers/specs/2026-05-21-tauri-anonymous-account-design.md` — read before starting; this plan implements it.

**Repo paths:**
- Rust core: `nym-vpn-core/crates/`
- Tauri app: `nym-vpn-app/` (frontend in `src/`, backend in `src-tauri/`)

**Build / check commands:**
- Rust core (from `nym-vpn-core/`): `cargo +nightly clippy -- -Dwarnings && cargo test`
- Rust formatting: `cargo +nightly fmt`
- Tauri app frontend (from `nym-vpn-app/`): `npm run check` (lint + tscheck + fmt:check)
- Tauri app types regen (from `nym-vpn-app/`): `npm run tsgen` (runs `cargo test` first to refresh ts-rs output)
- Full Tauri dev build (from `nym-vpn-app/`): `npm run dev:app`

---

## File map

**Rust core — modified:**
- `nym-vpn-core/crates/nym-vpn-store/src/types.rs` — extend `StorableAccount` with 3 flags.
- `nym-vpn-core/crates/nym-vpn-store/src/account/mod.rs` — extend `StoredAccount` + `From` impl; add `update_account` to trait.
- `nym-vpn-core/crates/nym-vpn-store/src/account/on_disk.rs` — implement `update_account` (read-modify-write).
- `nym-vpn-core/crates/nym-vpn-store/src/account/ephemeral.rs` — implement `update_account` in the in-memory store too.
- `nym-vpn-core/crates/nym-vpn-account-controller/src/storage/account.rs` — add `AccountStorageOp::UpdateAccountFlags` + `GetStoredMnemonic`; handle in dispatch loop.
- `nym-vpn-core/crates/nym-vpn-account-controller/src/commands/dispatch.rs` — add `AccountCommand::GetStoredMnemonic` + `AccountCommand::ConfirmMnemonicBackup`; handle in error path.
- `nym-vpn-core/crates/nym-vpn-account-controller/src/commands/handler.rs` — make `handle_register_anonymous_account` idempotent; set `is_locally_generated` flag in `handle_create_account`; add `handle_get_stored_mnemonic`, `handle_confirm_mnemonic_backup`.
- `nym-vpn-core/crates/nym-vpn-account-controller/src/command_sender.rs` — add `get_stored_mnemonic()` and `confirm_mnemonic_backup()` wrappers.
- `nym-vpn-core/crates/nym-vpn-lib/src/service/vpn_service.rs` — un-gate `RegisterAnonymousAccount` cfg to include `linux`; add `GetStoredMnemonic` + `ConfirmMnemonicBackup` variants (Linux only).
- `nym-vpn-core/crates/nym-vpn-proto/proto/nym_vpn_service.proto` — add 4 RPCs + `GetStoredMnemonicResponse` message; extend `AccountControllerState` with 3 booleans.
- `nym-vpn-core/crates/nym-vpn-proto/src/rpc_client.rs` — add 4 client methods.
- `nym-vpn-core/crates/nym-vpnd/src/command_interface.rs` (or wherever the gRPC service is implemented — confirm during impl) — implement the 4 RPC handlers; polkit per-call check on Linux for `GetStoredMnemonic`.
- `nym-vpn-core/crates/nym-vpn-lib-types/src/account/mod.rs` (or wherever `AccountControllerState` is defined with `ts_rs::TS` derive — confirm during impl) — add 3 new boolean fields.
- `nym-vpn-core/crates/nym-ipc/src/authentication/linux.rs` — extract the polkit-policy-install pattern into a reusable helper, and add a second action const for `reveal-mnemonic`. (Or duplicate the pattern in `command_interface.rs` — implementation choice; prefer extraction.)

**Tauri app — modified:**
- `nym-vpn-app/src-tauri/src/commands/account.rs` — add 4 commands `#[cfg(target_os = "linux")]`.
- `nym-vpn-app/src-tauri/src/vpnd/account.rs` — add 4 vpnd wrappers.
- `nym-vpn-app/src-tauri/src/error.rs` — add `MnemonicRevealDenied` + `MnemonicNotAvailable` to `ErrorKey`.
- `nym-vpn-app/src-tauri/src/main.rs` — register new commands in `invoke_handler` inside `#[cfg(target_os = "linux")]` block.
- `nym-vpn-app/src/types/tauri.ts` — auto-regenerated; do not hand-edit.
- `nym-vpn-app/src/store/slices/createMainSlice.ts` — store + selectors for 3 new account flags.
- `nym-vpn-app/src/screens/welcome/components/Signup.tsx` — Linux branch in `handleCreateAccount` calls local-create.
- `nym-vpn-app/src/screens/home/Home.tsx` — render new `MnemonicBackupBanner`; rework connect button per `useConnectButtonMode` hook.
- `nym-vpn-app/src/screens/settings/account/Account.tsx` — new "Recovery phrase" row.
- `nym-vpn-app/src/router.tsx` — add `revealMnemonic` route.
- `nym-vpn-app/src/types/routes.ts` (currently just a type alias; route table lives in `router.tsx`) — no change required.

**Tauri app — new:**
- `nym-vpn-app/src/hooks/useIsLinux.ts` — platform helper.
- `nym-vpn-app/src/screens/home/components/MnemonicBackupBanner.tsx` — banner component.
- `nym-vpn-app/src/screens/settings/account/RevealMnemonic.tsx` — reveal page.
- `nym-vpn-app/src/i18n/en/recovery-phrase.json` — new i18n namespace.

---

## Phase 1: Storage schema (Rust core)

### Task 1.1: Add 3 flags to `StorableAccount`

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpn-store/src/types.rs`
- Test: same file (existing `#[cfg(test)] mod tests`)

- [ ] **Step 1: Write the failing test**

Append to `nym-vpn-core/crates/nym-vpn-store/src/types.rs`:

```rust
#[cfg(test)]
mod flag_tests {
    use super::*;

    fn mnemonic() -> bip39::Mnemonic {
        "kiwi ketchup mix canvas curve ribbon congress method feel frozen act annual aunt comfort side joy mesh palace tennis cannon orange name tortoise piece".parse().unwrap()
    }

    #[test]
    fn new_storable_account_defaults_flags_to_false() {
        let a = StorableAccount::new(mnemonic(), StoredAccountMode::Api);
        assert!(!a.is_locally_generated);
        assert!(!a.is_registered_with_api);
        assert!(!a.is_backup_confirmed);
    }

    #[test]
    fn locally_generated_constructor_sets_flag() {
        let a = StorableAccount::new_locally_generated(mnemonic(), StoredAccountMode::Api);
        assert!(a.is_locally_generated);
        assert!(!a.is_registered_with_api);
        assert!(!a.is_backup_confirmed);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run (from `nym-vpn-core/`): `cargo test -p nym-vpn-store types::flag_tests -- --nocapture`
Expected: FAIL (`is_locally_generated` field missing on `StorableAccount`).

- [ ] **Step 3: Implement — extend `StorableAccount` and add `new_locally_generated`**

Replace the `StorableAccount` block in `nym-vpn-core/crates/nym-vpn-store/src/types.rs` with:

```rust
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct StorableAccount {
    pub mnemonic: bip39::Mnemonic,
    pub mode: StoredAccountMode,
    #[serde(default)]
    pub is_locally_generated: bool,
    #[serde(default)]
    pub is_registered_with_api: bool,
    #[serde(default)]
    pub is_backup_confirmed: bool,
}

impl StorableAccount {
    pub fn new(mnemonic: bip39::Mnemonic, mode: StoredAccountMode) -> StorableAccount {
        StorableAccount {
            mnemonic,
            mode,
            is_locally_generated: false,
            is_registered_with_api: false,
            is_backup_confirmed: false,
        }
    }

    pub fn new_locally_generated(
        mnemonic: bip39::Mnemonic,
        mode: StoredAccountMode,
    ) -> StorableAccount {
        StorableAccount {
            mnemonic,
            mode,
            is_locally_generated: true,
            is_registered_with_api: false,
            is_backup_confirmed: false,
        }
    }
}
```

Update the `Debug` impl to include the new fields (verbatim copy of field names is fine since they aren't secret).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p nym-vpn-store types::flag_tests -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Run full crate tests to catch fallout**

Run: `cargo test -p nym-vpn-store`
Expected: PASS (the `account_fixture` and on-disk tests construct `StorableAccount` via the struct literal — since the new fields have `#[serde(default)]` and the fixtures still compile, no fallout. If a fixture errors with "missing fields", update it to use `..Default::default()` after deriving `Default`, OR change to `StorableAccount::new(...)`.)

- [ ] **Step 6: Commit**

```bash
git add nym-vpn-core/crates/nym-vpn-store/src/types.rs
git commit -m "feat(store): add is_locally_generated/is_registered_with_api/is_backup_confirmed to StorableAccount

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task 1.2: Persist the 3 flags in `StoredAccount` (on-disk shape)

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpn-store/src/account/mod.rs`
- Test: same file

- [ ] **Step 1: Write the failing test**

Append a new test inside the existing `#[cfg(test)] pub(crate) mod test_fixtures` block (or a sibling `tests` block):

```rust
#[cfg(test)]
mod stored_account_tests {
    use super::*;

    #[test]
    fn stored_account_round_trips_new_flags() {
        let mnemonic = test_fixtures::mnemonic_fixture();
        let stored = StoredAccount {
            name: "default".to_string(),
            mnemonic: mnemonic.clone(),
            mode: StoredAccountMode::Api,
            nonce: 0,
            is_locally_generated: true,
            is_registered_with_api: true,
            is_backup_confirmed: false,
        };
        let storable: StorableAccount = stored.into();
        assert!(storable.is_locally_generated);
        assert!(storable.is_registered_with_api);
        assert!(!storable.is_backup_confirmed);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p nym-vpn-store account::stored_account_tests`
Expected: FAIL (`StoredAccount` doesn't have the new fields).

- [ ] **Step 3: Implement — extend `StoredAccount` and the `From` impl**

In `nym-vpn-core/crates/nym-vpn-store/src/account/mod.rs`, replace the `StoredAccount` struct and `From` impl with:

```rust
#[derive(Serialize, Deserialize)]
struct StoredAccount {
    /// Identifier of the account.
    name: String,

    /// The mnemonic itself.
    mnemonic: Mnemonic,

    /// The mode associated with this account
    /// note that it won't exist for legacy data
    #[serde(default)]
    mode: StoredAccountMode,

    /// Nonce used to confirm the mnemonic
    nonce: Nonce,

    /// True when the mnemonic was generated locally via CreateAccount (vs imported).
    #[serde(default)]
    is_locally_generated: bool,

    /// True after a successful registration with nym-vpn-api.
    #[serde(default)]
    is_registered_with_api: bool,

    /// True after the user has confirmed they saved the recovery phrase.
    #[serde(default)]
    is_backup_confirmed: bool,
}

impl std::fmt::Debug for StoredAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredAccount")
            .field("name", &self.name)
            .field("mnemonic", &"[redacted]")
            .field("mode", &self.mode)
            .field("nonce", &self.nonce)
            .field("is_locally_generated", &self.is_locally_generated)
            .field("is_registered_with_api", &self.is_registered_with_api)
            .field("is_backup_confirmed", &self.is_backup_confirmed)
            .finish()
    }
}

impl From<StoredAccount> for StorableAccount {
    fn from(account: StoredAccount) -> Self {
        StorableAccount {
            mnemonic: account.mnemonic,
            mode: account.mode,
            is_locally_generated: account.is_locally_generated,
            is_registered_with_api: account.is_registered_with_api,
            is_backup_confirmed: account.is_backup_confirmed,
        }
    }
}
```

- [ ] **Step 4: Update the on-disk writer to copy the new fields**

In `nym-vpn-core/crates/nym-vpn-store/src/account/on_disk.rs`, find `store_account` and update the `StoredAccount` construction:

```rust
let stored_account = StoredAccount {
    name,
    mnemonic: account.mnemonic,
    mode: account.mode,
    nonce,
    is_locally_generated: account.is_locally_generated,
    is_registered_with_api: account.is_registered_with_api,
    is_backup_confirmed: account.is_backup_confirmed,
};
```

(`StoredAccount` is `pub(super)`-ish — check visibility; if it's only used in `mod.rs`, you may need to make the fields `pub(super)` or expose a constructor. Simplest: make the fields `pub(crate)`.)

- [ ] **Step 5: Run all storage tests**

Run: `cargo test -p nym-vpn-store`
Expected: PASS. The legacy-mnemonic round-trip test still passes (new fields default to false during deserialization).

- [ ] **Step 6: Commit**

```bash
git add nym-vpn-core/crates/nym-vpn-store/src/account/mod.rs nym-vpn-core/crates/nym-vpn-store/src/account/on_disk.rs
git commit -m "feat(store): persist new account flags in on-disk StoredAccount

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task 1.3: Add `update_account` to the `AccountInformationStorage` trait

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpn-store/src/account/mod.rs`
- Modify: `nym-vpn-core/crates/nym-vpn-store/src/account/on_disk.rs`
- Modify: `nym-vpn-core/crates/nym-vpn-store/src/account/ephemeral.rs`
- Test: `nym-vpn-core/crates/nym-vpn-store/src/account/on_disk.rs` (new test)

- [ ] **Step 1: Write the failing test**

Append to the `tests` mod in `nym-vpn-core/crates/nym-vpn-store/src/account/on_disk.rs`:

```rust
#[tokio::test]
async fn update_account_sets_flags() {
    let account = account_fixture();
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("test.txt");
    let storage = OnDiskAccountStorage::new(path);
    storage.store_account(account.clone()).await.unwrap();

    storage
        .update_account(|a| {
            a.is_registered_with_api = true;
            a.is_backup_confirmed = true;
        })
        .await
        .unwrap();

    let loaded = storage.load_account().await.unwrap().unwrap();
    assert!(loaded.is_registered_with_api);
    assert!(loaded.is_backup_confirmed);
    assert!(!loaded.is_locally_generated);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p nym-vpn-store account::on_disk::tests::update_account_sets_flags`
Expected: FAIL (no `update_account` method on trait).

- [ ] **Step 3: Add `update_account` to the trait**

In `nym-vpn-core/crates/nym-vpn-store/src/account/mod.rs`, extend `AccountInformationStorage`:

```rust
#[async_trait::async_trait]
pub trait AccountInformationStorage {
    type StorageError: Error + Send + Sync + 'static;

    async fn load_account(&self) -> Result<Option<StorableAccount>, Self::StorageError>;
    async fn store_account(&self, account: StorableAccount) -> Result<(), Self::StorageError>;
    async fn remove_account(&self) -> Result<(), Self::StorageError>;
    async fn is_account_stored(&self) -> Result<bool, Self::StorageError> {
        self.load_account()
            .await
            .map(|maybe_account| maybe_account.is_some())
    }

    /// Apply an in-place mutation to the stored account and persist the result.
    /// The mutator must not change `mnemonic` or `mode` — only the flag fields
    /// (`is_locally_generated`, `is_registered_with_api`, `is_backup_confirmed`).
    async fn update_account(
        &self,
        mutator: Box<dyn FnOnce(&mut StorableAccount) + Send>,
    ) -> Result<(), Self::StorageError>;
}
```

(If `Box<dyn FnOnce>` causes lifetime grief with `async_trait`, fall back to a `with_account` pattern: `async fn update_account<F: FnOnce(&mut StorableAccount) + Send>(&self, f: F) -> ...`. Pick whichever the trait infrastructure accepts cleanly.)

- [ ] **Step 4: Implement in `OnDiskAccountStorage`**

In `nym-vpn-core/crates/nym-vpn-store/src/account/on_disk.rs`, append to the `impl AccountInformationStorage for OnDiskAccountStorage` block:

```rust
async fn update_account(
    &self,
    mutator: Box<dyn FnOnce(&mut StorableAccount) + Send>,
) -> Result<(), OnDiskMnemonicStorageError> {
    let mut account = self
        .load_account()
        .await?
        .ok_or_else(|| OnDiskMnemonicStorageError::FileOpenError(
            std::io::Error::new(std::io::ErrorKind::NotFound, "no account stored"),
        ))?;
    mutator(&mut account);

    // Read-modify-write: delete the existing file then re-store (existing
    // store_account rejects pre-existing files).
    self.remove_account().await?;
    self.store_account(account).await
}
```

- [ ] **Step 5: Implement in `EphemeralAccountStorage`**

In `nym-vpn-core/crates/nym-vpn-store/src/account/ephemeral.rs`, find the `impl AccountInformationStorage for EphemeralAccountStorage` block and append:

```rust
async fn update_account(
    &self,
    mutator: Box<dyn FnOnce(&mut StorableAccount) + Send>,
) -> Result<(), Self::StorageError> {
    let mut guard = self.account.lock().await;  // adjust to the actual field name/lock type
    let account = guard
        .as_mut()
        .ok_or(/* whatever the ephemeral "not stored" error is */)?;
    mutator(account);
    Ok(())
}
```

Verify the actual field name and error type by reading the file first. The shape above is correct in spirit but field names will differ.

- [ ] **Step 6: Update the legacy round-trip test if it now fails**

The `account_fixture` in `mod.rs` constructs `StorableAccount` via struct literal. If you didn't already add `..Default::default()` or derive `Default`, you may need to do it now. Easiest: change the fixture to use `StorableAccount::new(mnemonic, StoredAccountMode::Api)`.

- [ ] **Step 7: Run all storage tests**

Run: `cargo test -p nym-vpn-store`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add nym-vpn-core/crates/nym-vpn-store/src/account/
git commit -m "feat(store): add update_account method for in-place flag mutation

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase 2: Account-controller commands

### Task 2.1: Add storage ops for update + get-mnemonic

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpn-account-controller/src/storage/account.rs`

- [ ] **Step 1: Extend `AccountStorageOp` enum**

In `nym-vpn-core/crates/nym-vpn-account-controller/src/storage/account.rs` at the enum definition (around line 141):

```rust
pub(crate) enum AccountStorageOp {
    GetStoredAccount(ReturnSender<Option<StorableAccount>, Error>),
    StoreAccount(ReturnSender<Device, Error>, StorableAccount),
    ForgetAccount(ReturnSender<(), Error>),
    ResetKeys(ReturnSender<Device, Error>, Option<[u8; 32]>),
    UpdateAccountFlags(
        ReturnSender<(), Error>,
        Box<dyn FnOnce(&mut StorableAccount) + Send>,
    ),
}
```

- [ ] **Step 2: Handle the new op in `handle_storage_op`**

In the same file (around line 123):

```rust
pub(crate) async fn handle_storage_op(&self, op: AccountStorageOp) {
    match op {
        AccountStorageOp::GetStoredAccount(result_tx) => {
            result_tx.send(self.load_stored_account().await)
        }
        AccountStorageOp::StoreAccount(result_tx, account) => {
            result_tx.send(self.init_account(account).await)
        }
        AccountStorageOp::ForgetAccount(result_tx) => {
            result_tx.send(self.forget_account().await)
        }
        AccountStorageOp::ResetKeys(result_tx, seed) => {
            result_tx.send(self.reset_and_load_keys(seed).await)
        }
        AccountStorageOp::UpdateAccountFlags(result_tx, mutator) => {
            result_tx.send(
                self.mnemonic_storage
                    .update_account(mutator)
                    .await
                    .map_err(|e| Error::Storage(e.to_string())),
            )
        }
    }
}
```

Verify the field name `self.mnemonic_storage` matches the file (it might be `self.account_storage` or similar — read the top of the impl).

- [ ] **Step 3: Build to confirm it compiles**

Run: `cargo build -p nym-vpn-account-controller`
Expected: PASS (no new tests yet; we just wired the op).

- [ ] **Step 4: Commit**

```bash
git add nym-vpn-core/crates/nym-vpn-account-controller/src/storage/account.rs
git commit -m "feat(account-controller): add UpdateAccountFlags storage op

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task 2.2: Add `GetStoredMnemonic` + `ConfirmMnemonicBackup` to AccountCommand

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpn-account-controller/src/commands/dispatch.rs`

- [ ] **Step 1: Add new variants**

In `nym-vpn-core/crates/nym-vpn-account-controller/src/commands/dispatch.rs`, add to the `AccountCommand` enum near the other variants:

```rust
/// Read the stored mnemonic phrase out of the secure store. Caller is responsible
/// for any authentication gating (e.g. polkit at the gRPC boundary).
GetStoredMnemonic(ReturnSender<String, AccountCommandError>),

/// Mark the recovery phrase as backed up by the user. Idempotent.
ConfirmMnemonicBackup(ReturnSender<(), AccountCommandError>),
```

- [ ] **Step 2: Handle them in `handle_command_error`**

In the same file's error-propagation match block (around line 71-90):

```rust
AccountCommand::GetStoredMnemonic(return_sender) => return_sender.send(Err(error)),
AccountCommand::ConfirmMnemonicBackup(return_sender) => return_sender.send(Err(error)),
```

- [ ] **Step 3: Build**

Run: `cargo build -p nym-vpn-account-controller`
Expected: FAIL — match-must-be-exhaustive against `AccountCommand` somewhere else in the crate. Find that match (likely in the main dispatch loop in `commands/mod.rs` or `lib.rs`) and add stub arms calling the handlers from Task 2.3 (write the dispatch arms now; handlers come next):

```rust
AccountCommand::GetStoredMnemonic(tx) => {
    let _ = tx.send(crate::commands::handler::handle_get_stored_mnemonic(&mut self.shared_state).await);
}
AccountCommand::ConfirmMnemonicBackup(tx) => {
    let _ = tx.send(crate::commands::handler::handle_confirm_mnemonic_backup(&mut self.shared_state).await);
}
```

The exact `&mut self.shared_state` reference may differ — match the pattern of existing arms in the same file.

- [ ] **Step 4: Build to confirm it still fails (waiting on handlers)**

Run: `cargo build -p nym-vpn-account-controller`
Expected: FAIL with "no function `handle_get_stored_mnemonic` in module `handler`" — that's the cue for Task 2.3.

- [ ] **Step 5: Commit (will not compile yet — leave the WIP commit for clean history)**

Skip the commit; bundle with Task 2.3.

### Task 2.3: Add handler bodies + make `handle_register_anonymous_account` idempotent + set `is_locally_generated` in create/store

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpn-account-controller/src/commands/handler.rs`

- [ ] **Step 1: Write failing tests**

Append a `#[cfg(test)] mod handler_tests` block (use the existing test infrastructure in the crate as a reference — there may already be a `tests` mod with shared fixtures; reuse if so):

```rust
#[cfg(test)]
mod handler_tests {
    use super::*;
    // ... import test fixtures the crate provides (look at existing tests in this file or sibling test files)

    #[tokio::test]
    async fn handle_get_stored_mnemonic_returns_phrase() {
        let mut shared = /* build a SharedAccountState with a stored mnemonic via the test harness */;
        let phrase = handle_get_stored_mnemonic(&mut shared).await.unwrap();
        assert_eq!(phrase.split_whitespace().count(), 24);
    }

    #[tokio::test]
    async fn handle_confirm_mnemonic_backup_sets_flag() {
        let mut shared = /* build state with an account where is_backup_confirmed = false */;
        handle_confirm_mnemonic_backup(&mut shared).await.unwrap();
        let account = /* re-fetch via storage_op_sender GetStoredAccount */;
        assert!(account.is_backup_confirmed);
    }

    #[tokio::test]
    async fn register_anonymous_account_short_circuits_when_already_registered() {
        let mut shared = /* state with is_registered_with_api = true */;
        let call_count_before = /* read the mocked api client call counter */;
        handle_register_anonymous_account(&mut shared, /* account */).await.unwrap();
        let call_count_after = /* same */;
        assert_eq!(call_count_before, call_count_after);
    }
}
```

If the crate lacks a mock `VpnApiClient`, this becomes harder — in that case fall back to testing in `vpn_service.rs` integration tests, and skip the unit assertions on `register_anonymous_account` idempotency. **Read existing handler tests first** to gauge the available test scaffolding before writing the above.

- [ ] **Step 2: Run tests to confirm they fail**

Run: `cargo test -p nym-vpn-account-controller commands::handler::handler_tests`
Expected: FAIL (functions don't exist / fixtures don't compile).

- [ ] **Step 3: Implement `handle_get_stored_mnemonic`**

Append to `nym-vpn-core/crates/nym-vpn-account-controller/src/commands/handler.rs`:

```rust
pub(crate) async fn handle_get_stored_mnemonic<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<String, AccountCommandError> {
    let (tx, rx) = ReturnSender::new();
    shared_state
        .storage_op_sender
        .send(AccountStorageOp::GetStoredAccount(tx))
        .map_err(AccountCommandError::internal)?;
    let account = rx
        .await
        .map_err(AccountCommandError::internal)?
        .map_err(AccountCommandError::storage)?
        .ok_or(AccountCommandError::NoAccountStored)?;
    Ok(account.mnemonic.to_string())
}
```

Verify `AccountCommandError::NoAccountStored` exists — if not, add it to the error enum in `nym-vpn-lib-types`. If the enum is `non_exhaustive`-ish, add the variant; if the wire format matters, follow the existing convention used by the Android-side `handle_register_anonymous_account`.

- [ ] **Step 4: Implement `handle_confirm_mnemonic_backup`**

```rust
pub(crate) async fn handle_confirm_mnemonic_backup<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<(), AccountCommandError> {
    let (tx, rx) = ReturnSender::new();
    shared_state
        .storage_op_sender
        .send(AccountStorageOp::UpdateAccountFlags(
            tx,
            Box::new(|a| a.is_backup_confirmed = true),
        ))
        .map_err(AccountCommandError::internal)?;
    rx.await
        .map_err(AccountCommandError::internal)?
        .map_err(AccountCommandError::storage)
}
```

- [ ] **Step 5: Make `handle_register_anonymous_account` idempotent + set flag on success**

Find the existing function (added on the Android branch) at `nym-vpn-core/crates/nym-vpn-account-controller/src/commands/handler.rs`. Replace its body with:

```rust
pub(crate) async fn handle_register_anonymous_account<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    account: StorableAccount,
) -> Result<RegisterAccountResponse, AccountCommandError> {
    // Idempotent: short-circuit if already registered.
    if account.is_registered_with_api {
        tracing::debug!("Anonymous account already registered; no-op");
        return Ok(RegisterAccountResponse {
            account_token: String::new(),
        });
    }

    let vpn_account = VpnAccount::try_from(account.clone())
        .map_err(|e| AccountCommandError::InvalidMnemonic(e.to_string()))?;
    let _ = shared_state
        .vpn_api_client
        .register_anonymous_account(&vpn_account)
        .await?;

    // Persist is_registered_with_api = true. Best-effort: log on storage error
    // but still return Ok since the API call succeeded.
    let (tx, rx) = ReturnSender::new();
    if shared_state
        .storage_op_sender
        .send(AccountStorageOp::UpdateAccountFlags(
            tx,
            Box::new(|a| a.is_registered_with_api = true),
        ))
        .is_ok()
    {
        if let Err(e) = rx.await {
            tracing::error!("Failed to receive storage update result: {e}");
        }
    }

    tracing::debug!("Anonymous account registered with API");

    Ok(RegisterAccountResponse {
        account_token: String::new(),
    })
}
```

- [ ] **Step 6: Set `is_locally_generated` in `handle_create_account`**

Replace the `StorableAccount::new(...)` call inside `handle_create_account` with `StorableAccount::new_locally_generated(...)`:

```rust
.send(AccountStorageOp::StoreAccount(
    tx,
    StorableAccount::new_locally_generated(mnemonic, vpn_account.mode().into()),
))
```

`handle_store_account` already gets the `StorableAccount` from the caller (which uses `StorableAccount::new()` → `is_locally_generated = false`). No change needed.

- [ ] **Step 7: Run handler tests**

Run: `cargo test -p nym-vpn-account-controller commands::handler`
Expected: PASS (or the tests you skipped due to scaffolding gaps in Step 1 are left out).

- [ ] **Step 8: Build the whole crate**

Run: `cargo build -p nym-vpn-account-controller && cargo +nightly clippy -p nym-vpn-account-controller -- -Dwarnings`
Expected: PASS.

- [ ] **Step 9: Commit**

```bash
git add nym-vpn-core/crates/nym-vpn-account-controller/
git commit -m "feat(account-controller): add GetStoredMnemonic + ConfirmMnemonicBackup commands; make RegisterAnonymousAccount idempotent

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task 2.4: Add command_sender wrappers

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpn-account-controller/src/command_sender.rs`

- [ ] **Step 1: Add wrappers**

After the existing `register_anonymous_account` wrapper (added on the Android branch), append:

```rust
#[instrument(skip(self))]
pub async fn get_stored_mnemonic(&self) -> Result<String, AccountCommandError> {
    let (tx, rx) = ReturnSender::new();
    self.command_tx
        .send(AccountCommand::GetStoredMnemonic(tx))
        .map_err(AccountCommandError::internal)?;
    rx.await.map_err(AccountCommandError::internal)?
}

#[instrument(skip(self))]
pub async fn confirm_mnemonic_backup(&self) -> Result<(), AccountCommandError> {
    let (tx, rx) = ReturnSender::new();
    self.command_tx
        .send(AccountCommand::ConfirmMnemonicBackup(tx))
        .map_err(AccountCommandError::internal)?;
    rx.await.map_err(AccountCommandError::internal)?
}
```

- [ ] **Step 2: Build**

Run: `cargo build -p nym-vpn-account-controller`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add nym-vpn-core/crates/nym-vpn-account-controller/src/command_sender.rs
git commit -m "feat(account-controller): expose get_stored_mnemonic and confirm_mnemonic_backup on AccountCommandSender

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase 3: vpn-service layer

### Task 3.1: Un-gate `RegisterAnonymousAccount` to include Linux

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpn-lib/src/service/vpn_service.rs`

- [ ] **Step 1: Replace the cfg attribute on the `VpnServiceCommand` variant**

In `nym-vpn-core/crates/nym-vpn-lib/src/service/vpn_service.rs`, find:

```rust
#[cfg(any(target_os = "android", target_os = "ios"))]
RegisterAnonymousAccount(
    oneshot::Sender<Result<RegisterAccountResponse, AccountCommandError>>,
    (),
),
```

Replace the cfg with:

```rust
#[cfg(any(target_os = "android", target_os = "ios", target_os = "linux"))]
```

Apply the same change to the corresponding match arm in `handle_command` and the `handle_register_anonymous_account` method body.

- [ ] **Step 2: Build**

Run: `cargo build -p nym-vpn-lib`
Expected: PASS on Linux.

- [ ] **Step 3: Commit**

```bash
git add nym-vpn-core/crates/nym-vpn-lib/src/service/vpn_service.rs
git commit -m "feat(vpn-service): enable RegisterAnonymousAccount on Linux

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task 3.2: Add `GetStoredMnemonic` + `ConfirmMnemonicBackup` to VpnServiceCommand (Linux only)

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpn-lib/src/service/vpn_service.rs`

- [ ] **Step 1: Add new VpnServiceCommand variants**

Near the existing `RegisterAnonymousAccount` variant:

```rust
#[cfg(target_os = "linux")]
GetStoredMnemonic(
    oneshot::Sender<Result<String, AccountCommandError>>,
    (),
),
#[cfg(target_os = "linux")]
ConfirmMnemonicBackup(
    oneshot::Sender<Result<(), AccountCommandError>>,
    (),
),
```

- [ ] **Step 2: Add match arms in `handle_command`**

Near the existing `RegisterAnonymousAccount` arm:

```rust
#[cfg(target_os = "linux")]
VpnServiceCommand::GetStoredMnemonic(tx, ()) => {
    let _ = tx.send(self.handle_get_stored_mnemonic().await);
}
#[cfg(target_os = "linux")]
VpnServiceCommand::ConfirmMnemonicBackup(tx, ()) => {
    let _ = tx.send(self.handle_confirm_mnemonic_backup().await);
}
```

- [ ] **Step 3: Implement the handler methods**

Near `handle_register_anonymous_account` (which is at line ~1803):

```rust
#[cfg(target_os = "linux")]
async fn handle_get_stored_mnemonic(&self) -> Result<String, AccountCommandError> {
    self.account_command_tx.get_stored_mnemonic().await
}

#[cfg(target_os = "linux")]
async fn handle_confirm_mnemonic_backup(&self) -> Result<(), AccountCommandError> {
    self.account_command_tx.confirm_mnemonic_backup().await
}
```

- [ ] **Step 4: Build**

Run: `cargo build -p nym-vpn-lib`
Expected: PASS on Linux. On Windows the new code is cfg-gated out — verify by `cargo build --target x86_64-pc-windows-gnu` if the target is installed, otherwise rely on Phase 12 manual check.

- [ ] **Step 5: Commit**

```bash
git add nym-vpn-core/crates/nym-vpn-lib/src/service/vpn_service.rs
git commit -m "feat(vpn-service): add GetStoredMnemonic and ConfirmMnemonicBackup VpnServiceCommand variants (Linux)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task 3.3: Add UniFFI surface (no-op for Tauri; keep parity for Android)

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpn-lib-uniffi/src/vpn_service_command_sender.rs`

- [ ] **Step 1: Decide whether to expose**

The UniFFI bindings already expose `register_anonymous_account` (added on the Android branch). `GetStoredMnemonic` and `ConfirmMnemonicBackup` are Tauri-only — do **not** expose them via UniFFI. Skip this task entirely if no changes are needed.

- [ ] **Step 2: Build to confirm UniFFI side is unchanged**

Run: `cargo build -p nym-vpn-lib-uniffi`
Expected: PASS (no change).

No commit needed if no files changed.

---

## Phase 4: Daemon proto + gRPC handlers

### Task 4.1: Extend the proto with 4 new RPCs + flags on AccountControllerState

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpn-proto/proto/nym_vpn_service.proto`

- [ ] **Step 1: Add the response message**

After the existing `message AccountCommandResponse { ... }` block (~line 812):

```proto
message GetStoredMnemonicResponse {
  string mnemonic = 1;
}
```

- [ ] **Step 2: Add the 4 RPCs**

Inside `service NymVpnService { ... }`, right after the existing `rpc StoreAccount(...)` declaration (line ~1193):

```proto
// Generate a new mnemonic and store it locally. Does NOT register with the
// nym-vpn-api backend.
rpc CreateAccount(google.protobuf.Empty) returns (AccountCommandResponse) {}

// Register the locally-stored account with the nym-vpn-api. Idempotent:
// no-op if already registered.
rpc RegisterAnonymousAccount(google.protobuf.Empty) returns (AccountCommandResponse) {}

// Reveal the stored recovery phrase. On Linux the daemon performs a per-call
// polkit authentication check; on other platforms returns Unimplemented.
rpc GetStoredMnemonic(google.protobuf.Empty) returns (GetStoredMnemonicResponse) {}

// Mark the recovery phrase as backed up by the user.
rpc ConfirmMnemonicBackup(google.protobuf.Empty) returns (google.protobuf.Empty) {}
```

- [ ] **Step 3: Extend `AccountControllerState` (or the relevant state message) with 3 booleans**

Find the message returned by `GetAccountState` (likely `AccountControllerState`). Add:

```proto
bool is_locally_generated = N;       // pick next free tag
bool is_registered_with_api = N + 1;
bool is_backup_confirmed = N + 2;
```

The exact tag numbers depend on the current last-used tag; pick the next free ones.

- [ ] **Step 4: Regenerate proto bindings**

Run: `cargo build -p nym-vpn-proto`
Expected: PASS — the build script regenerates Rust types from the proto. If it fails, investigate the build script error.

- [ ] **Step 5: Commit**

```bash
git add nym-vpn-core/crates/nym-vpn-proto/proto/nym_vpn_service.proto
git commit -m "proto: add CreateAccount, RegisterAnonymousAccount, GetStoredMnemonic, ConfirmMnemonicBackup RPCs

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task 4.2: Wire the 4 RPCs into `RpcClient`

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpn-proto/src/rpc_client.rs`

- [ ] **Step 1: Add client methods**

After the existing `store_account` method, add four wrappers in the `impl RpcClient` block:

```rust
pub async fn create_account(&mut self) -> Result<AccountCommandResponse> {
    let response = self
        .0
        .create_account(())
        .await
        .map_err(Error::Rpc)?
        .into_inner();
    AccountCommandResponse::try_from(response).map_err(Error::InvalidResponse)
}

pub async fn register_anonymous_account(&mut self) -> Result<AccountCommandResponse> {
    let response = self
        .0
        .register_anonymous_account(())
        .await
        .map_err(Error::Rpc)?
        .into_inner();
    AccountCommandResponse::try_from(response).map_err(Error::InvalidResponse)
}

pub async fn get_stored_mnemonic(&mut self) -> Result<String> {
    let response = self
        .0
        .get_stored_mnemonic(())
        .await
        .map_err(Error::Rpc)?
        .into_inner();
    Ok(response.mnemonic)
}

pub async fn confirm_mnemonic_backup(&mut self) -> Result<()> {
    self.0
        .confirm_mnemonic_backup(())
        .await
        .map_err(Error::Rpc)?;
    Ok(())
}
```

`AccountCommandResponse` already implements `TryFrom` for proto responses based on existing patterns — match those.

- [ ] **Step 2: Build**

Run: `cargo build -p nym-vpn-proto`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add nym-vpn-core/crates/nym-vpn-proto/src/rpc_client.rs
git commit -m "feat(proto-client): add 4 new account RPC wrappers

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task 4.3: Implement daemon-side gRPC handlers (non-Linux + Linux without polkit, polkit added in Phase 5)

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpnd/src/command_interface.rs` (or whichever file contains `impl NymVpnService for ...`)

- [ ] **Step 1: Locate the service impl**

Run: `grep -rn 'impl NymVpnService for\|impl nym_vpn_service::NymVpnService for' nym-vpn-core/crates/nym-vpnd/`. Note the file path for the next steps.

- [ ] **Step 2: Add `create_account`, `register_anonymous_account`, `confirm_mnemonic_backup` handlers**

In the service impl, add three new async methods (signatures auto-generated by tonic from the proto):

```rust
async fn create_account(
    &self,
    _request: Request<()>,
) -> Result<Response<AccountCommandResponse>, Status> {
    let result = self.service_command_sender.create_account().await;
    Ok(Response::new(map_account_command_response(result)))
}

async fn register_anonymous_account(
    &self,
    _request: Request<()>,
) -> Result<Response<AccountCommandResponse>, Status> {
    #[cfg(any(target_os = "android", target_os = "ios", target_os = "linux"))]
    {
        let result = self.service_command_sender.register_anonymous_account().await;
        Ok(Response::new(map_account_command_response(result)))
    }
    #[cfg(not(any(target_os = "android", target_os = "ios", target_os = "linux")))]
    {
        Err(Status::unimplemented("register_anonymous_account is not supported on this platform"))
    }
}

async fn confirm_mnemonic_backup(
    &self,
    _request: Request<()>,
) -> Result<Response<()>, Status> {
    #[cfg(target_os = "linux")]
    {
        self.service_command_sender
            .confirm_mnemonic_backup()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(()))
    }
    #[cfg(not(target_os = "linux"))]
    {
        Err(Status::unimplemented("confirm_mnemonic_backup is Linux-only"))
    }
}
```

`self.service_command_sender` and `map_account_command_response` are pseudo-names — match what the existing handlers use (e.g. `store_account` handler is a good template).

- [ ] **Step 3: Add `get_stored_mnemonic` handler (without polkit yet — placeholder)**

```rust
async fn get_stored_mnemonic(
    &self,
    _request: Request<()>,
) -> Result<Response<GetStoredMnemonicResponse>, Status> {
    #[cfg(target_os = "linux")]
    {
        // TODO(phase 5): wrap with per-call polkit check.
        let mnemonic = self
            .service_command_sender
            .get_stored_mnemonic()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(GetStoredMnemonicResponse { mnemonic }))
    }
    #[cfg(not(target_os = "linux"))]
    {
        Err(Status::unimplemented("get_stored_mnemonic is Linux-only"))
    }
}
```

- [ ] **Step 4: Build the daemon**

Run: `cargo build -p nym-vpnd`
Expected: PASS on Linux.

- [ ] **Step 5: Commit**

```bash
git add nym-vpn-core/crates/nym-vpnd/src/
git commit -m "feat(vpnd): add gRPC handlers for new account RPCs (polkit TBD)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task 4.4: Expose the 3 new account-state flags through the daemon's `GetAccountState`

**Files:**
- Modify: wherever `AccountControllerState` is mapped from internal Rust → proto in the daemon (typically `nym-vpnd/src/command_interface.rs` or a `into_proto` helper).
- Modify: `nym-vpn-core/crates/nym-vpn-lib-types/src/account/mod.rs` (or wherever `AccountControllerState` is defined with `ts_rs::TS`).

- [ ] **Step 1: Locate the type definition**

Run: `grep -rln 'AccountControllerState' nym-vpn-core/crates/nym-vpn-lib-types/`. Open the file and find the struct with the ts_rs derive.

- [ ] **Step 2: Add the 3 fields**

Add to the struct:

```rust
pub is_locally_generated: bool,
pub is_registered_with_api: bool,
pub is_backup_confirmed: bool,
```

- [ ] **Step 3: Populate them in the daemon-side mapper**

Find where `AccountControllerState` is constructed from the in-memory state (look for `AccountControllerState {` literal — there's at least one place that builds the response). Read the current `SharedAccountState`'s stored account (via a `GetStoredAccount` op, or directly if accessible) and copy the three booleans through.

- [ ] **Step 4: Update the proto→types mapping in `rpc_client.rs`**

If `AccountControllerState` has a manual `TryFrom<proto::AccountControllerState>` impl, add the three fields to the conversion. If it's auto-derived (e.g. via prost), nothing to do — just confirm `cargo build -p nym-vpn-proto` passes.

- [ ] **Step 5: Build**

Run: `cargo build -p nym-vpn-lib-types -p nym-vpn-proto -p nym-vpnd`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add nym-vpn-core/
git commit -m "feat(lib-types,vpnd): surface new account flags on AccountControllerState

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase 5: Polkit per-call gate on `GetStoredMnemonic` (Linux)

### Task 5.1: Add the `reveal-mnemonic` polkit action constant + install-on-first-use

**Files:**
- Modify: `nym-vpn-core/crates/nym-ipc/src/authentication/linux.rs` (extract reusable helper)
- Or modify: `nym-vpn-core/crates/nym-vpnd/src/command_interface.rs` (call polkit directly)

Prefer extraction — gives a clean reusable API.

- [ ] **Step 1: Add a public helper to `nym-ipc`**

In `nym-vpn-core/crates/nym-ipc/src/authentication/linux.rs`, after `wait_for_authorization`, add:

```rust
/// Request per-call polkit authorization for an arbitrary action id.
/// Installs the policy file on first use if it doesn't already exist.
///
/// Returns Ok if the user authenticated, Err otherwise (denied / timed out / cancelled).
pub async fn request_action_authorization(
    cred: nix::sys::socket::UnixCredentials,
    action_id: &str,
    policy_xml: &str,
    shutdown_token: CancellationToken,
) -> Result<(), AuthenticationError> {
    let connection = Connection::system()
        .await
        .map_err(AuthenticationError::MessageBusConnection)?;
    let proxy = AuthorityProxy::new(&connection)
        .await
        .map_err(AuthenticationError::AuthorityProxy)?;

    // Install policy if missing (mirrors the existing pattern in AuthProxy::check_authorization).
    if !proxy
        .enumerate_actions("")
        .await
        .map_err(AuthenticationError::EnumerateActions)?
        .iter()
        .any(|a| a.action_id == action_id)
    {
        let path = format!("/usr/share/polkit-1/actions/{action_id}.policy");
        let mut file = tokio::fs::File::create(&path)
            .await
            .map_err(AuthenticationError::CreateActionPolicy)?;
        file.write_all(policy_xml.as_bytes())
            .await
            .map_err(AuthenticationError::WriteActionPolicy)?;
    }

    let subject = Subject::new_for_owner(
        cred.pid().try_into().map_err(AuthenticationError::NumberConversion)?,
        None,
        Some(cred.uid()),
    )
    .map_err(AuthenticationError::Subject)?;

    let auth_proxy = AuthProxy { proxy, subject };
    let timeout = tokio::time::sleep(USER_INTERACTION_TIMEOUT);
    let auth_result = wait_for_authorization(auth_proxy, shutdown_token, timeout).await?;

    if auth_result.is_authorized {
        Ok(())
    } else {
        Err(AuthenticationError::AuthorizationDenied)
    }
}
```

(`AuthProxy` is private — either make it `pub(crate)` and re-export, or inline its logic. The above assumes it's accessible.)

- [ ] **Step 2: Export from `nym-ipc/src/lib.rs`**

Re-export the helper:

```rust
#[cfg(target_os = "linux")]
pub use authentication::linux::request_action_authorization;
```

- [ ] **Step 3: Add the `reveal-mnemonic` action const in `nym-vpnd`**

In the same daemon module as the gRPC service impl, add:

```rust
#[cfg(target_os = "linux")]
const REVEAL_MNEMONIC_ACTION_ID: &str = "com.nymvpn.vpnd.reveal-mnemonic";

#[cfg(target_os = "linux")]
const REVEAL_MNEMONIC_POLICY: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<policyconfig>
  <action id="com.nymvpn.vpnd.reveal-mnemonic">
    <description>Reveal stored recovery phrase</description>
    <message>Authentication is required to reveal the recovery phrase</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_self</allow_active>
    </defaults>
  </action>
</policyconfig>
"#;
```

- [ ] **Step 4: Wire the polkit check into `get_stored_mnemonic`**

Replace the body of `get_stored_mnemonic` in the daemon's gRPC service impl:

```rust
async fn get_stored_mnemonic(
    &self,
    request: Request<()>,
) -> Result<Response<GetStoredMnemonicResponse>, Status> {
    #[cfg(target_os = "linux")]
    {
        // Extract the peer credentials of the calling tonic request.
        let cred = request
            .extensions()
            .get::<nix::sys::socket::UnixCredentials>()
            .copied()
            .ok_or_else(|| Status::unauthenticated("no peer credentials available"))?;

        nym_ipc::request_action_authorization(
            cred,
            REVEAL_MNEMONIC_ACTION_ID,
            REVEAL_MNEMONIC_POLICY,
            self.shutdown_token.clone(),
        )
        .await
        .map_err(|e| Status::permission_denied(e.to_string()))?;

        let mnemonic = self
            .service_command_sender
            .get_stored_mnemonic()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(GetStoredMnemonicResponse { mnemonic }))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = request;
        Err(Status::unimplemented("get_stored_mnemonic is Linux-only"))
    }
}
```

`UnixCredentials` must be propagated into the tonic request extensions by the IPC layer. Check `nym-ipc/src/uds.rs` — if peer credentials aren't yet attached, add them to the request extensions in the auth layer. If wiring credentials through tonic extensions proves nontrivial, fall back to `getsockopt(stream, PeerCredentials)` inside the polkit helper — but that requires access to the underlying `UnixStream`, which tonic abstracts away. The cleanest path is to attach `UnixCredentials` as an extension during connection setup; if not feasible in this PR, document the limitation and use a fixed `Subject::new_for_owner(self_pid, None, None)` (the daemon's own pid + uid), which still triggers polkit but binds to the daemon's identity rather than the client's. Confirm during implementation which path is feasible.

- [ ] **Step 5: Build the daemon on Linux**

Run: `cargo build -p nym-vpnd --target x86_64-unknown-linux-gnu`
Expected: PASS.

- [ ] **Step 6: Unit-test the new helper using the existing MockProxy pattern**

In `nym-vpn-core/crates/nym-ipc/src/authentication/linux.rs`, add a test for `request_action_authorization`'s success and denial paths. Reuse the `MockProxy` / `MockPrompter` infrastructure that already exists in the test module. The shape:

```rust
#[tokio::test]
async fn request_action_authorization_authorized_path() {
    // Build a MockProxy that returns is_authorized = true.
    // Stub out the policy-install branch (the test harness should not write to /usr/share).
    // Assert request_action_authorization returns Ok.
}
```

If the helper writes the policy file to a path that isn't easily mockable, you may need to refactor the helper to take the policy-install closure as a parameter for testability. Skip if it adds disproportionate complexity — the integration test in Phase 12 will exercise it.

- [ ] **Step 7: Commit**

```bash
git add nym-vpn-core/crates/nym-ipc/ nym-vpn-core/crates/nym-vpnd/
git commit -m "feat(vpnd): per-call polkit gate on GetStoredMnemonic (Linux)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase 6: Tauri Rust backend

### Task 6.1: Add vpnd wrappers

**Files:**
- Modify: `nym-vpn-app/src-tauri/src/vpnd/account.rs`

- [ ] **Step 1: Add wrappers**

In `nym-vpn-app/src-tauri/src/vpnd/account.rs`, after the existing `store_account` method, add four `#[cfg(target_os = "linux")]` wrappers (gate only the Linux-only ones; `create_account` and `register_anonymous_account` can stay unconditional since the proto exposes them — but since we won't call them from non-Linux, gating keeps the code path explicit):

```rust
#[cfg(target_os = "linux")]
pub async fn create_account(&self) -> Result<(), VpndError> {
    let mut vpnd = self.vpnd().await?;
    vpnd.create_account()
        .or_else(async |e| self.handle_rpc_error("create_account", e).await)
        .await
        .map(|_| ())
}

#[cfg(target_os = "linux")]
pub async fn register_anonymous_account(&self) -> Result<(), VpndError> {
    let mut vpnd = self.vpnd().await?;
    vpnd.register_anonymous_account()
        .or_else(async |e| self.handle_rpc_error("register_anonymous_account", e).await)
        .await
        .map(|_| ())
}

#[cfg(target_os = "linux")]
pub async fn get_stored_mnemonic(&self) -> Result<String, VpndError> {
    let mut vpnd = self.vpnd().await?;
    vpnd.get_stored_mnemonic()
        .or_else(async |e| self.handle_rpc_error("get_stored_mnemonic", e).await)
        .await
}

#[cfg(target_os = "linux")]
pub async fn confirm_mnemonic_backup(&self) -> Result<(), VpndError> {
    let mut vpnd = self.vpnd().await?;
    vpnd.confirm_mnemonic_backup()
        .or_else(async |e| self.handle_rpc_error("confirm_mnemonic_backup", e).await)
        .await
}
```

`self.vpnd()` returns the locked `RpcClient` per the existing pattern at line ~96 of `vpnd/client.rs`. Pattern-match the exact return type of existing `store_account` to keep the error-handling shape identical.

- [ ] **Step 2: Build**

Run (from `nym-vpn-app/src-tauri/`): `cargo build`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add nym-vpn-app/src-tauri/src/vpnd/account.rs
git commit -m "feat(tauri): add vpnd wrappers for 4 new account RPCs

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task 6.2: Add Tauri commands

**Files:**
- Modify: `nym-vpn-app/src-tauri/src/commands/account.rs`
- Modify: `nym-vpn-app/src-tauri/src/error.rs`
- Modify: `nym-vpn-app/src-tauri/src/main.rs`

- [ ] **Step 1: Extend `ErrorKey`**

In `nym-vpn-app/src-tauri/src/error.rs`, add two variants to `ErrorKey`:

```rust
MnemonicRevealDenied,
MnemonicNotAvailable,
```

If `ErrorKey` is `#[derive(ts_rs::TS)]`, the new variants will be picked up by the next `npm run tsgen`.

- [ ] **Step 2: Map gRPC `permission_denied` → `MnemonicRevealDenied`**

In `nym-vpn-app/src-tauri/src/error.rs` (or wherever `BackendError::from` for `VpndError` lives), add a branch that maps `tonic::Code::PermissionDenied` from `get_stored_mnemonic` to `BackendError::new("…", ErrorKey::MnemonicRevealDenied)`. If the existing mapping is too generic to selectively detect "the call was get_stored_mnemonic", introduce a small wrapping error in `vpnd::account` (e.g. return a `Result<String, VpndError>` plus a known wrapper that the command-layer interprets). Keep it simple — a custom `match` in `commands/account.rs::get_stored_mnemonic` is fine:

```rust
#[cfg(target_os = "linux")]
#[instrument(skip_all)]
#[tauri::command]
pub async fn get_stored_mnemonic(vpnd: State<'_, VpndClient>) -> Result<String, BackendError> {
    vpnd.get_stored_mnemonic().await.map_err(|e| {
        if matches!(&e, VpndError::RpcClient(rpc_e) if rpc_e.is_permission_denied()) {
            BackendError::new("polkit authentication failed", ErrorKey::MnemonicRevealDenied)
        } else {
            error!("failed to get stored mnemonic: {e}");
            e.into()
        }
    })
}
```

(`is_permission_denied` may not exist on `rpc_client::Error` — if not, pattern-match `Error::Rpc(status) if status.code() == tonic::Code::PermissionDenied`.)

- [ ] **Step 3: Add the other 3 commands**

In `nym-vpn-app/src-tauri/src/commands/account.rs`, all gated:

```rust
#[cfg(target_os = "linux")]
#[instrument(skip_all)]
#[tauri::command]
pub async fn create_local_account(
    vpnd: State<'_, VpndClient>,
    app_state: State<'_, SharedAppState>,
) -> Result<(), BackendError> {
    let state = app_state.lock().await;
    if !matches!(state.tunnel, TunnelState::Disconnected) {
        return Err(BackendError::internal(
            &format!("cannot create account from state {}", state.tunnel),
            None,
        ));
    }
    drop(state);

    vpnd.create_account().await.map_err(|e| {
        error!("failed to create local account: {e}");
        e.into()
    })
}

#[cfg(target_os = "linux")]
#[instrument(skip_all)]
#[tauri::command]
pub async fn register_anonymous_account(
    vpnd: State<'_, VpndClient>,
) -> Result<(), BackendError> {
    vpnd.register_anonymous_account().await.map_err(|e| {
        error!("failed to register anonymous account: {e}");
        e.into()
    })
}

#[cfg(target_os = "linux")]
#[instrument(skip_all)]
#[tauri::command]
pub async fn confirm_mnemonic_backup(
    vpnd: State<'_, VpndClient>,
) -> Result<(), BackendError> {
    vpnd.confirm_mnemonic_backup().await.map_err(|e| {
        error!("failed to confirm mnemonic backup: {e}");
        e.into()
    })
}
```

- [ ] **Step 4: Register in `invoke_handler`**

In `nym-vpn-app/src-tauri/src/main.rs`, find the `tauri::generate_handler!` call (or the `invoke_handler` registration). Add the 4 new commands inside a `#[cfg(target_os = "linux")]` block:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    #[cfg(target_os = "linux")]
    cmd_account::create_local_account,
    #[cfg(target_os = "linux")]
    cmd_account::register_anonymous_account,
    #[cfg(target_os = "linux")]
    cmd_account::get_stored_mnemonic,
    #[cfg(target_os = "linux")]
    cmd_account::confirm_mnemonic_backup,
])
```

`generate_handler!` may not accept `#[cfg]` directly inside the macro on all Tauri versions. If it doesn't, build two separate invoke-handler builders — one with the base set, one with the +Linux additions — and select at compile time via `cfg`. Pattern:

```rust
let builder = tauri::Builder::default();
#[cfg(target_os = "linux")]
let builder = builder.invoke_handler(tauri::generate_handler![
    // base + Linux extras
]);
#[cfg(not(target_os = "linux"))]
let builder = builder.invoke_handler(tauri::generate_handler![
    // base only
]);
```

- [ ] **Step 5: Build the Tauri Rust backend**

Run (from `nym-vpn-app/`): `cd src-tauri && cargo build`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add nym-vpn-app/src-tauri/
git commit -m "feat(tauri): add 4 new account commands (Linux only)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase 7: ts-rs regen + Zustand wiring

### Task 7.1: Regenerate TypeScript types

**Files:**
- Modify: `nym-vpn-app/src/types/tauri.ts` (auto)

- [ ] **Step 1: Run tsgen**

Run (from `nym-vpn-app/`): `npm run tsgen`
Expected: success; `src/types/tauri.ts` now contains the 3 new boolean fields on `TAccountControllerState` (or whichever type maps from `AccountControllerState`) and the 2 new `MnemonicRevealDenied` / `MnemonicNotAvailable` variants in the error key enum.

- [ ] **Step 2: Confirm by `git diff`**

Run: `git diff src/types/tauri.ts`
Expected: new fields appear.

- [ ] **Step 3: Run `npm run check`**

Run: `npm run check`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add nym-vpn-app/src/types/tauri.ts
git commit -m "chore(tauri): regenerate TypeScript types

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task 7.2: Add platform helper hook

**Files:**
- Create: `nym-vpn-app/src/hooks/useIsLinux.ts`
- Modify: `nym-vpn-app/src/hooks/index.ts` (re-export)

- [ ] **Step 1: Check what's already available for platform detection**

Run: `grep -rn 'platform\|os_type\|os.platform\|navigator.platform' nym-vpn-app/src/`. There's likely an existing Tauri command (`get_platform`?) or a `platform` value in the main Zustand slice. If yes: write the hook as a wrapper around that. If no: create a Tauri command `get_os` returning `'linux' | 'windows' | 'macos'` and have the hook call it once via `useEffect`, caching in a `useRef`.

- [ ] **Step 2: Implement the hook**

```typescript
// nym-vpn-app/src/hooks/useIsLinux.ts
import { useAppStore } from '../store';

export function useIsLinux(): boolean {
  // If a `platform` value already exists in the store, use it. Otherwise see Step 1.
  const platform = useAppStore((s) => s.platform);
  return platform === 'linux';
}
```

Adjust to whatever store/store-action returns the platform.

- [ ] **Step 3: Export from `hooks/index.ts`**

```typescript
export { useIsLinux } from './useIsLinux';
```

- [ ] **Step 4: Run check**

Run: `npm run check`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add nym-vpn-app/src/hooks/
git commit -m "feat(tauri): add useIsLinux hook

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task 7.3: Extend Zustand main slice with the 3 new account flags

**Files:**
- Modify: `nym-vpn-app/src/store/slices/createMainSlice.ts`

- [ ] **Step 1: Add fields and selectors**

In the slice's state interface and action handler that processes `AccountControllerState`:

```typescript
type AccountFlags = {
  isLocallyGenerated: boolean;
  isRegisteredWithApi: boolean;
  isBackupConfirmed: boolean;
};

// In MainSlice state:
accountFlags: AccountFlags;

// Default state:
accountFlags: {
  isLocallyGenerated: false,
  isRegisteredWithApi: false,
  isBackupConfirmed: false,
},

// In the reducer that handles 'set-account-state' (or the equivalent action that
// receives the AccountControllerState from a Tauri event):
case 'set-account-state':
  return {
    ...state,
    accountState: action.state,
    accountFlags: {
      isLocallyGenerated: action.state.is_locally_generated,
      isRegisteredWithApi: action.state.is_registered_with_api,
      isBackupConfirmed: action.state.is_backup_confirmed,
    },
  };
```

The action shape depends on the existing convention (look at the existing `set-account-state` or whichever action mutates `accountState`).

- [ ] **Step 2: Add convenience selectors**

```typescript
export const useAccountLocallyGenerated = () => useAppStore((s) => s.accountFlags.isLocallyGenerated);
export const useAccountRegistered = () => useAppStore((s) => s.accountFlags.isRegisteredWithApi);
export const useAccountBackupConfirmed = () => useAppStore((s) => s.accountFlags.isBackupConfirmed);
```

Place these in the same file as other store-derived hooks.

- [ ] **Step 3: Run check**

Run: `npm run check`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add nym-vpn-app/src/store/
git commit -m "feat(tauri): track new account flags in Zustand main slice

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase 8: Frontend — welcome flow change

### Task 8.1: Rewire `Signup.tsx` to use `create_local_account` on Linux

**Files:**
- Modify: `nym-vpn-app/src/screens/welcome/components/Signup.tsx`

- [ ] **Step 1: Replace `handleCreateAccount` body**

Open `nym-vpn-app/src/screens/welcome/components/Signup.tsx`. Replace the `handleCreateAccount` callback body with:

```typescript
const handleCreateAccount = async () => {
  if (isLinux) {
    try {
      await invoke('create_local_account');
      dispatch({ type: 'set-account', stored: true });
      await CCache.del('cache-account-id');
      await CCache.del('cache-device-id');
      dispatch({ type: 'reset-error' });
      handleNavigate();
    } catch (error) {
      console.error('[Signup] create_local_account failed:', error);
      // Surface as toast — reuse existing error-toast helper used elsewhere on this screen.
      // No navigation on failure.
    }
    return;
  }

  // Non-Linux: existing web-deeplink flow (verbatim from current implementation).
  const url = await invoke<string>('get_deep_link', {
    locale: i18n.language,
    kind: 'CreateAccount',
  });
  openUrl(url);
  try {
    const deeplinkurl = await startListening(600000);
    await invoke('store_deeplink_account', { callbackUrl: deeplinkurl });
    dispatch({ type: 'set-account', stored: true });
    await CCache.del('cache-account-id');
    await CCache.del('cache-device-id');
    dispatch({ type: 'reset-error' });
    handleNavigate();
  } catch (error) {
    console.error('[Signup] Create account error: ', error);
    handleNavigate();
  }
};
```

Add `const isLinux = useIsLinux();` at the top of the component along with the existing hook calls.

- [ ] **Step 2: Confirm no other call sites need updating**

Run: `grep -rn 'create_local_account\|get_deep_link.*CreateAccount' nym-vpn-app/src/`. Only `Signup.tsx` should match.

- [ ] **Step 3: Run check**

Run: `npm run check`
Expected: PASS.

- [ ] **Step 4: Build + manual sanity (skip if you'll batch with Phase 12)**

Run: `npm run dev:app`. Hit "Sign up anonymously" — on Linux, lands on Home immediately; no browser opens.

- [ ] **Step 5: Commit**

```bash
git add nym-vpn-app/src/screens/welcome/components/Signup.tsx
git commit -m "feat(tauri): welcome Sign up anonymously generates local account on Linux

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase 9: Frontend — reveal page + route + i18n

### Task 9.1: Add the `recovery-phrase` i18n namespace (English)

**Files:**
- Create: `nym-vpn-app/src/i18n/en/recovery-phrase.json`
- Modify: i18n init file that lists namespaces (likely `src/i18n/index.ts` — confirm during impl)

- [ ] **Step 1: Create the JSON file**

```json
{
  "title": "Recovery phrase",
  "warning": "Anyone with this phrase can access your account. Keep it private.",
  "reveal-button": "Reveal",
  "auth-denied-toast": "Authentication was cancelled or denied.",
  "copy-button": "Copy",
  "copied-toast": "Copied to clipboard",
  "saved-checkbox": "I have saved my recovery phrase in a safe place",
  "continue-button": "Continue",
  "back-button": "Back"
}
```

- [ ] **Step 2: Register namespace**

In the i18n init file, append `'recovery-phrase'` to the namespace list.

- [ ] **Step 3: Add the row label + banner copy to existing namespaces**

In `nym-vpn-app/src/i18n/en/settings.json`, add under the `account` key:

```json
"recovery-phrase": "Recovery phrase"
```

In `nym-vpn-app/src/i18n/en/home.json` (or whichever existing home namespace there is — if none, create `home.json`):

```json
"backup-banner": {
  "title": "Save your recovery phrase",
  "description": "It's the only way to recover your account. Save it now.",
  "action": "Reveal"
},
"get-plan": {
  "button": "Get a plan",
  "error": "Could not start checkout. Try again."
}
```

- [ ] **Step 4: Run check**

Run: `npm run check`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add nym-vpn-app/src/i18n/
git commit -m "feat(tauri): add i18n strings for recovery-phrase namespace and home banner

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task 9.2: Add the `revealMnemonic` route

**Files:**
- Modify: `nym-vpn-app/src/router.tsx`

- [ ] **Step 1: Add the route constant**

In the `routes` object in `nym-vpn-app/src/router.tsx`:

```typescript
revealMnemonic: '/settings/account/recovery-phrase',
```

- [ ] **Step 2: Add the child route under `/settings`**

Find the `routes.accountSettings` route entry inside the `/settings` children array. Convert it to a route with its own `children`:

```typescript
{
  path: routes.accountSettings,
  Component: AccountScreen,
  errorElement: <Error />,
},
{
  path: routes.revealMnemonic,
  Component: RevealMnemonic,
  errorElement: <Error />,
},
```

(Sibling route — simplest. If we want nested routing instead, refactor `accountSettings` to have `index: true` + the reveal as a child. Sibling is fine for now.)

- [ ] **Step 3: Import `RevealMnemonic` from screens**

Add to `src/screens/index.ts`:

```typescript
export { RevealMnemonic } from './settings/account/RevealMnemonic';
```

(File will be created in Task 9.3 — leaving the import dangling now will fail the build; reorder these two tasks if it matters.)

- [ ] **Step 4: Commit (after Task 9.3 to keep CI green)**

Bundle with Task 9.3.

### Task 9.3: Implement `RevealMnemonic.tsx`

**Files:**
- Create: `nym-vpn-app/src/screens/settings/account/RevealMnemonic.tsx`

- [ ] **Step 1: Implement the page**

```tsx
import { useTranslation } from 'react-i18next';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { Button, MsIcon } from '../../../ui';
import { routes } from '../../../router';
import { useAccountLocallyGenerated, useAccountBackupConfirmed, dispatch } from '../../../store';
import { useIsLinux } from '../../../hooks';

type View = 'idle' | 'prompting' | 'revealed';

export function RevealMnemonic() {
  const { t } = useTranslation('recovery-phrase');
  const navigate = useNavigate();
  const isLinux = useIsLinux();
  const isLocallyGenerated = useAccountLocallyGenerated();
  const isBackupConfirmed = useAccountBackupConfirmed();

  const [view, setView] = useState<View>('idle');
  const [mnemonic, setMnemonic] = useState<string | undefined>(undefined);
  const [checked, setChecked] = useState(false);
  const [confirming, setConfirming] = useState(false);

  // Memory hygiene: drop mnemonic on unmount.
  useEffect(() => {
    return () => {
      setMnemonic(undefined);
    };
  }, []);

  // Non-Linux users should not reach this page. Defensive guard.
  if (!isLinux) {
    navigate(routes.accountSettings);
    return null;
  }

  const handleReveal = async () => {
    setView('prompting');
    try {
      const phrase = await invoke<string>('get_stored_mnemonic');
      setMnemonic(phrase);
      setView('revealed');
    } catch (error: any) {
      console.warn('[RevealMnemonic] reveal denied or failed:', error);
      // Toast — reuse the existing toast helper used elsewhere.
      // toast.error(t('auth-denied-toast'));
      setView('idle');
    }
  };

  const handleCopy = async () => {
    if (mnemonic) {
      await writeText(mnemonic);
      // toast.success(t('copied-toast'));
    }
  };

  const handleConfirm = async () => {
    setConfirming(true);
    try {
      await invoke('confirm_mnemonic_backup');
      // Force a state refresh so the banner clears immediately.
      await invoke('refresh_account_state');
      navigate(routes.accountSettings);
    } catch (error) {
      console.error('[RevealMnemonic] confirm failed:', error);
    } finally {
      setConfirming(false);
    }
  };

  const handleBack = () => {
    setMnemonic(undefined);
    navigate(routes.accountSettings);
  };

  const showBackupCheckbox = isLocallyGenerated && !isBackupConfirmed && view === 'revealed';

  return (
    <div className="flex h-full flex-col gap-6 p-6">
      <h1 className="text-text-primary text-2xl font-medium">{t('title')}</h1>

      <div className="border-cheddar text-cheddar bg-cheddar/10 flex items-center gap-3 rounded-lg border p-3">
        <MsIcon icon="report" />
        <p>{t('warning')}</p>
      </div>

      {view === 'idle' && (
        <Button onClick={handleReveal}>{t('reveal-button')}</Button>
      )}

      {view === 'prompting' && (
        <div className="text-text-secondary">{/* spinner */}…</div>
      )}

      {view === 'revealed' && mnemonic && (
        <>
          <div className="grid grid-cols-3 gap-2">
            {mnemonic.split(/\s+/).map((word, i) => (
              <div
                key={i}
                className="bg-bg-secondary text-text-primary rounded p-2 text-center"
              >
                <span className="text-text-secondary mr-2 text-xs">{i + 1}.</span>
                {word}
              </div>
            ))}
          </div>
          <Button onClick={handleCopy}>{t('copy-button')}</Button>

          {showBackupCheckbox && (
            <>
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={checked}
                  onChange={(e) => setChecked(e.target.checked)}
                />
                <span>{t('saved-checkbox')}</span>
              </label>
              <Button onClick={handleConfirm} disabled={!checked || confirming}>
                {t('continue-button')}
              </Button>
            </>
          )}
        </>
      )}

      <Button onClick={handleBack} variant="secondary">
        {t('back-button')}
      </Button>
    </div>
  );
}
```

Notes:
- `writeText` from `@tauri-apps/plugin-clipboard-manager` — verify the plugin is installed in `package.json`. If not, install it: `npm i @tauri-apps/plugin-clipboard-manager` and add to `Cargo.toml` of `src-tauri`.
- `refresh_account_state` Tauri command must exist (it probably does — `grep -n refresh_account_state nym-vpn-app/src-tauri/src/`). If not, create one that calls `vpnd.refresh_account_state()`.
- `MsIcon`, `Button`, `dispatch` paths — copy from a sibling settings screen (e.g. `Account.tsx`) for the exact import paths.
- Tailwind class names above are best-effort; match the project's design tokens by looking at an existing settings page like `Account.tsx`.

- [ ] **Step 2: Run check**

Run: `npm run check`
Expected: PASS. Lint may complain about commented-out toast helpers — wire to the actual toast hook the project uses (look at error handling in `Signup.tsx`).

- [ ] **Step 3: Commit (bundle with Task 9.2)**

```bash
git add nym-vpn-app/src/router.tsx nym-vpn-app/src/screens/
git commit -m "feat(tauri): add Reveal recovery phrase page and route

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task 9.4: Add the Settings → Account → Recovery phrase row

**Files:**
- Modify: `nym-vpn-app/src/screens/settings/account/Account.tsx`

- [ ] **Step 1: Read the existing file to understand the row pattern**

Read `nym-vpn-app/src/screens/settings/account/Account.tsx`. Note how other rows (e.g. autologin, device ID) are rendered using `AccountSettingRow`.

- [ ] **Step 2: Add the new row**

Add a new `AccountSettingRow` for "Recovery phrase". Visibility: `isLinux && accountStored`. The row's `onClick` navigates to `routes.revealMnemonic`:

```tsx
const isLinux = useIsLinux();
const accountStored = useAppStore((s) => s.account);
// ...

{isLinux && accountStored && (
  <AccountSettingRow
    label={t('account.recovery-phrase')}
    onClick={() => navigate(routes.revealMnemonic)}
    icon="chevron_right"
  />
)}
```

Match the exact prop shape `AccountSettingRow` expects.

- [ ] **Step 3: Run check**

Run: `npm run check`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add nym-vpn-app/src/screens/settings/account/Account.tsx
git commit -m "feat(tauri): add Recovery phrase row in Settings → Account

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase 10: Frontend — home backup banner

### Task 10.1: Implement `MnemonicBackupBanner.tsx`

**Files:**
- Create: `nym-vpn-app/src/screens/home/components/MnemonicBackupBanner.tsx`

- [ ] **Step 1: Implement the banner**

```tsx
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { MsIcon, Button } from '../../../ui';
import { routes } from '../../../router';
import {
  useAccountLocallyGenerated,
  useAccountBackupConfirmed,
  useAppStore,
} from '../../../store';
import { useIsLinux } from '../../../hooks';

export function MnemonicBackupBanner() {
  const { t } = useTranslation('home');
  const navigate = useNavigate();
  const isLinux = useIsLinux();
  const isLocallyGenerated = useAccountLocallyGenerated();
  const isBackupConfirmed = useAccountBackupConfirmed();
  const accountState = useAppStore((s) => s.accountState);

  const show =
    isLinux &&
    accountState === 'Ready' &&
    isLocallyGenerated &&
    !isBackupConfirmed;

  if (!show) return null;

  return (
    <div className="border-cheddar bg-cheddar/10 text-cheddar mb-4 flex items-center gap-3 rounded-lg border p-3">
      <MsIcon icon="report" />
      <div className="flex-1">
        <p className="font-medium">{t('backup-banner.title')}</p>
        <p className="text-sm">{t('backup-banner.description')}</p>
      </div>
      <Button onClick={() => navigate(routes.revealMnemonic)} variant="text">
        {t('backup-banner.action')}
      </Button>
    </div>
  );
}
```

- [ ] **Step 2: Render in Home above the connect button**

Read `nym-vpn-app/src/screens/home/Home.tsx`. Identify where the connect button is rendered. Place `<MnemonicBackupBanner />` immediately above it.

- [ ] **Step 3: Run check**

Run: `npm run check`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add nym-vpn-app/src/screens/home/
git commit -m "feat(tauri): add MnemonicBackupBanner above connect button

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase 11: Frontend — Get-a-plan button

### Task 11.1: Render "Get a plan" instead of "Connect" when no active subscription

**Files:**
- Modify: `nym-vpn-app/src/screens/home/components/ConnectionButton.tsx` (or wherever the connect button lives — confirm by `grep`)

- [ ] **Step 1: Find the connect button component**

Run: `grep -rln 'ConnectTunnel\|invoke.*connect_tunnel\|Connect.*Button' nym-vpn-app/src/screens/home/`. Open the file.

- [ ] **Step 2: Add a mode discriminator**

At the top of the connect-button component, derive whether to render the "Get a plan" variant:

```typescript
const accountState = useAppStore((s) => s.accountState);
const accountStored = useAppStore((s) => s.account);
const isLinux = useIsLinux();

// "Get a plan" mode: stored account but no connectable state.
const needsPlan =
  isLinux &&
  accountStored &&
  accountState !== 'Ready'; // refine with the actual "can connect" set if it's broader
```

Refine the `accountState` check to match what the existing connect button uses to enable itself — look at the existing `disabled` condition on the connect button.

- [ ] **Step 3: Wire the new click handler**

```typescript
const { i18n, t } = useTranslation('home');

const handleGetPlan = async () => {
  setLoading(true);
  try {
    await invoke('register_anonymous_account');
    const url = await invoke<{ url: string }>('get_autologin_deeplink', {
      locale: i18n.language,
      kind: 'CreateAccount',
    });
    openUrl(url.url);
  } catch (e) {
    console.error('[ConnectionButton] get-plan failed:', e);
    // toast.error(t('get-plan.error'))
  } finally {
    setLoading(false);
  }
};

// Render switch:
if (needsPlan) {
  return (
    <Button onClick={handleGetPlan} loading={loading}>
      {t('get-plan.button')}
    </Button>
  );
}

// Otherwise: existing Connect / Disconnect button.
```

- [ ] **Step 4: Confirm Windows behaviour is unchanged**

The `needsPlan` derivation requires `isLinux === true`. On Windows it's always false, so the existing connect button renders as before.

- [ ] **Step 5: Run check**

Run: `npm run check`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add nym-vpn-app/src/screens/home/
git commit -m "feat(tauri): render 'Get a plan' button when stored account lacks subscription (Linux)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase 12: Manual QA + integration build

### Task 12.1: Full Linux QA pass

- [ ] **Step 1: Build the full app**

Run (from `nym-vpn-app/`): `npm run build:app`
Expected: PASS.

- [ ] **Step 2: Run the dev app against a real daemon**

Run: `npm run dev:app`

- [ ] **Step 3: Execute the QA checklist from the spec**

For each item, mark pass/fail in your dev notes:

1. Fresh install (no stored account) → Welcome → Sign up anonymously → lands on Home in stored state, no browser opens.
2. Home shows "Get a plan" button. Click → registration POST succeeds → browser opens to autologin URL. Banner not visible.
3. Simulate subscription activation (via DEV settings or test API) → `AccountState === 'Ready'` → banner appears above Connect button.
4. Settings → Account → Recovery phrase row visible. Click → reveal page. Click Reveal → polkit prompt appears → enter password → 24 words display + copy button + checkbox.
5. Tick checkbox → Continue → back to Settings → Account; banner gone on Home within ~1 s.
6. Re-open reveal page → polkit prompt appears again (mnemonic was dropped from memory). 24 words display, no checkbox (already confirmed).
7. With banner visible (not yet confirmed), open reveal page, press Back without ticking → banner persists; re-enter prompts polkit again.
8. Click Reveal → cancel polkit dialog → toast appears, page stays in idle, no navigation.
9. Settings → Forget account → reveal row hidden; welcome screen reachable.
10. Welcome → Login → PassphraseEnter with a real existing mnemonic → Reveal row visible in Settings → Account; no banner ever appears.
11. (If Privy account is reachable) login via Privy → Reveal row visible; revealed content displays (verify expected format per spec follow-up question).
12. Build Windows target (cross-compile or VM): no banner, no reveal row, "Sign up anonymously" still opens browser. Connect button behaviour unchanged.

- [ ] **Step 4: If any QA step fails**

Open a new debugging task. Fix in a small commit referencing the QA item number. Re-run the failing item.

### Task 12.2: Final type/lint sweep

- [ ] **Step 1: Rust core**

Run (from `nym-vpn-core/`): `cargo +nightly clippy -- -Dwarnings && cargo +nightly fmt --check && cargo test`
Expected: PASS.

- [ ] **Step 2: Tauri Rust**

Run (from `nym-vpn-app/src-tauri/`): `cargo +nightly clippy -- -Dwarnings && cargo +nightly fmt --check && cargo test`
Expected: PASS.

- [ ] **Step 3: Tauri frontend**

Run (from `nym-vpn-app/`): `npm run check`
Expected: PASS.

- [ ] **Step 4: If anything fails**

Fix in small commits; re-run from Step 1.

### Task 12.3: Final manual smoke on Linux + Windows builds

- [ ] **Step 1: Final Linux smoke**

`npm run dev:app` → Sign up anonymously → Get a plan → Reveal → confirm checkbox → re-open Reveal → polkit re-prompts → back → reveal again. Once each.

- [ ] **Step 2: Final Windows smoke (or VM, or cross-build target)**

Confirm Windows behaviour is unchanged: Sign up anonymously opens browser as before; Settings → Account has no recovery-phrase row; Home has no banner.

- [ ] **Step 3: Commit any final fixups**

```bash
git status
# any remaining changes
git commit -am "chore: final lint fixups"
```

---

## Follow-ups (out of scope for this plan)

- Resolve the Privy mnemonic format question with the user (see spec). If the daemon returns something other than a 24-word phrase, branch the `RevealMnemonic.tsx` renderer on `account.mode`.
- Windows port of the OS-auth gate (Windows Hello / UAC) for `GetStoredMnemonic`. Will let us remove the platform-gate on the reveal page on Windows.
- Translations beyond English: hand off `recovery-phrase.json` to crowdin per the existing localization process.

