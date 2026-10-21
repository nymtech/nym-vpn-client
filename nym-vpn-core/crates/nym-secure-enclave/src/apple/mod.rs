// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod auth;
mod security;
mod zeroing_allocator;

use std::{cell::RefCell, collections::HashMap, sync::Arc};

use objc2::rc::Retained;
use objc2_core_foundation::{CFAllocator, CFRetained};
use objc2_local_authentication::LAContext;
use objc2_security::{
    SecKey, errSecAuthFailed, errSecDataNotAvailable, errSecItemNotFound, errSecUserCanceled,
};
use tokio_util::task::LocalPoolHandle;
use zeroize::Zeroizing;

use self::{
    auth::{AuthErrorKind, authenticate, non_interactive_context},
    security::{SecError, create_sec_key, decrypt_data, delete_sec_key, encrypt_data, get_sec_key},
    zeroing_allocator::create_zeroing_allocator,
};
use crate::{Result, SecureEnclaveError, SecureEnclaveKey, SecureEnclaveProvider};

type KeyId = u64;

thread_local! {
    /// Keys owned by the worker thread. `SecKey` cannot be sent
    /// between threads, so it never leaves the thread it was created on.
    static WORKER_STATE: RefCell<WorkerState> = RefCell::default();
}

struct WorkerState {
    keys: HashMap<KeyId, KeyEntry>,
    next_key_id: KeyId,
    auth: AuthState,
}

struct KeyEntry {
    tag: Vec<u8>,
    key: CFRetained<SecKey>,
    /// Generation of the authentication context the key was obtained with.
    auth_generation: u64,
}

struct AuthState {
    /// Context that keys are obtained with. When unlocked, it is the context evaluated by
    /// `unlock()`. When locked, it does not allow user interaction, so that keys requiring user
    /// presence fail instead of prompting.
    context: Retained<LAContext>,
    /// Incremented whenever the context changes, so that keys bound to an older context are
    /// fetched again.
    generation: u64,
    /// Incremented on every `lock()`, so that an `unlock()` in flight does not install its
    /// context after the enclave was locked.
    lock_epoch: u64,
}

impl AuthState {
    fn locked() -> Self {
        Self {
            context: non_interactive_context(),
            generation: 0,
            lock_epoch: 0,
        }
    }

    fn unlock(&mut self, context: Retained<LAContext>) {
        self.set_context(context);
    }

    fn lock(&mut self) {
        self.lock_epoch += 1;
        self.set_context(non_interactive_context());
    }

    fn set_context(&mut self, context: Retained<LAContext>) {
        let old_context = std::mem::replace(&mut self.context, context);
        unsafe { old_context.invalidate() };
        self.generation += 1;
    }
}

impl Default for WorkerState {
    /// Runs once on the worker thread, before the first call into the Security framework.
    fn default() -> Self {
        // Set global default CFAllocator on current thread.
        match create_zeroing_allocator() {
            Some(allocator) => CFAllocator::set_default(Some(&allocator)),
            None => tracing::warn!("Failed to create zeroing CFAllocator"),
        }

        Self {
            keys: HashMap::new(),
            next_key_id: 0,
            auth: AuthState::locked(),
        }
    }
}

impl WorkerState {
    fn insert(&mut self, tag: Vec<u8>, key: CFRetained<SecKey>) -> KeyId {
        let key_id = self.next_key_id;
        self.next_key_id += 1;
        self.keys.insert(
            key_id,
            KeyEntry {
                tag,
                key,
                auth_generation: self.auth.generation,
            },
        );
        key_id
    }

    /// Returns the key, fetching it again if the authentication context changed since the key
    /// was obtained.
    fn key(&mut self, key_id: KeyId) -> Result<&CFRetained<SecKey>> {
        let entry = self
            .keys
            .get_mut(&key_id)
            .ok_or_else(|| SecureEnclaveError::Other("key is released".to_owned()))?;

        if entry.auth_generation != self.auth.generation {
            entry.key = get_sec_key(&entry.tag, &self.auth.context)?;
            entry.auth_generation = self.auth.generation;
        }

        Ok(&entry.key)
    }
}

pub struct AppleSecureEnclaveProvider {
    /// Pool with a single worker thread performing all calls into secure enclave.
    pool: LocalPoolHandle,
}

impl Default for AppleSecureEnclaveProvider {
    fn default() -> Self {
        Self {
            pool: LocalPoolHandle::new(1),
        }
    }
}

#[async_trait::async_trait]
impl SecureEnclaveProvider for AppleSecureEnclaveProvider {
    type Key = OSSecureEnclaveKey;

    async fn create_key(&self, alias: &str, user_presence: bool) -> Result<Self::Key> {
        let alias_bytes = alias.as_bytes().to_vec();
        let id = run_on_worker(&self.pool, move |state| {
            // Prevent creating more than one key with the same alias.
            match get_sec_key(&alias_bytes, &state.auth.context).map_err(SecureEnclaveError::from) {
                Ok(_) => Err(SecureEnclaveError::KeyAlreadyExists),
                Err(SecureEnclaveError::KeyNotFound) => {
                    let key = create_sec_key(&alias_bytes, user_presence, &state.auth.context)?;
                    Ok(state.insert(alias_bytes, key))
                }
                Err(err) => Err(err),
            }
        })
        .await?;
        Ok(OSSecureEnclaveKey::new(id, self.pool.clone()))
    }

    async fn delete_key(&self, alias: &str) -> Result<()> {
        let alias_bytes = alias.as_bytes().to_vec();
        run_on_worker(&self.pool, move |_state| Ok(delete_sec_key(&alias_bytes)?)).await
    }

    async fn get_key(&self, alias: &str) -> Result<Self::Key> {
        let alias_bytes = alias.as_bytes().to_vec();
        let id = run_on_worker(&self.pool, move |state| {
            let key = get_sec_key(&alias_bytes, &state.auth.context)?;
            Ok(state.insert(alias_bytes, key))
        })
        .await?;
        Ok(OSSecureEnclaveKey::new(id, self.pool.clone()))
    }

    async fn unlock(&self, reason: &str) -> Result<()> {
        let reason = reason.to_owned();
        self.pool
            .spawn_pinned(move || async move {
                let lock_epoch = WORKER_STATE.with_borrow(|state| state.auth.lock_epoch);
                let context = authenticate(&reason).await?;

                WORKER_STATE.with_borrow_mut(|state| {
                    if state.auth.lock_epoch == lock_epoch {
                        state.auth.unlock(context);
                        Ok(())
                    } else {
                        unsafe { context.invalidate() };
                        Err(SecureEnclaveError::UserCancelled)
                    }
                })
            })
            .await
            .map_err(|_| SecureEnclaveError::Other("secure enclave worker panic".to_owned()))?
    }

    async fn lock(&self) -> Result<()> {
        run_on_worker(&self.pool, |state| {
            state.auth.lock();
            Ok(())
        })
        .await
    }
}

#[derive(Clone)]
pub struct OSSecureEnclaveKey {
    handle: Arc<KeyHandle>,
}

impl OSSecureEnclaveKey {
    fn new(id: KeyId, pool: LocalPoolHandle) -> Self {
        Self {
            handle: Arc::new(KeyHandle { id, pool }),
        }
    }
}

#[async_trait::async_trait]
impl SecureEnclaveKey for OSSecureEnclaveKey {
    async fn decrypt(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let key_id = self.handle.id;
        let bytes = bytes.to_vec();

        run_on_worker(&self.handle.pool, move |state| {
            let private_key = state.key(key_id)?;
            Ok(decrypt_data(private_key, &bytes)?)
        })
        .await
    }

    async fn encrypt(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let key_id = self.handle.id;
        let bytes = Zeroizing::new(bytes.to_vec());

        run_on_worker(&self.handle.pool, move |state| {
            let private_key = state.key(key_id)?;
            Ok(encrypt_data(private_key, &bytes)?)
        })
        .await
    }
}

/// Reference to the key owned by the worker thread. The key is released once the last handle is
/// dropped.
struct KeyHandle {
    id: KeyId,
    pool: LocalPoolHandle,
}

impl Drop for KeyHandle {
    fn drop(&mut self) {
        let key_id = self.id;
        let _join_handle = self.pool.spawn_pinned(move || async move {
            WORKER_STATE.with_borrow_mut(|state| state.keys.remove(&key_id));
        });
    }
}

impl From<SecError> for SecureEnclaveError {
    fn from(error: SecError) -> Self {
        if error.is_interaction_not_allowed() {
            Self::Locked
        } else if let Some(os_status) = error.os_status() {
            match os_status {
                x if x == errSecAuthFailed => Self::AuthenticationFailed,
                x if x == errSecDataNotAvailable => Self::AccessDeniedUntilDeviceUnlocked,
                x if x == errSecUserCanceled => Self::UserCancelled,
                x if x == errSecItemNotFound => Self::KeyNotFound,
                _ => Self::Other(error.to_string()),
            }
        } else {
            match error {
                SecError::Os(_) | SecError::Cf(_) => Self::Other(error.to_string()),
                SecError::Auth(err) => match err.kind {
                    AuthErrorKind::Failed => Self::AuthenticationFailed,
                    AuthErrorKind::Cancelled => Self::UserCancelled,
                    AuthErrorKind::Other => Self::Other(err.to_string()),
                },
                SecError::Internal(msg) => Self::Other(msg.to_owned()),
            }
        }
    }
}

/// Run the closure on the worker thread and wait for it to complete.
async fn run_on_worker<T, F>(pool: &LocalPoolHandle, f: F) -> Result<T>
where
    F: FnOnce(&mut WorkerState) -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    pool.spawn_pinned(move || async move { WORKER_STATE.with_borrow_mut(f) })
        .await
        .map_err(|_| SecureEnclaveError::Other("secure enclave worker panic".to_owned()))?
}
