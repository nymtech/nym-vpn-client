// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{cell::RefCell, collections::HashMap, sync::Arc};

use tokio_util::task::LocalPoolHandle;
use zeroize::Zeroizing;

use crate::{Result, SecureEnclaveError, SecureEnclaveKey, SecureEnclaveProvider};

type KeyId = u64;

thread_local! {
    /// Keys owned by the worker thread. `secure_enclave_rs::SecureEnclaveKey` cannot be sent
    /// between threads, so it never leaves the thread it was created on.
    static WORKER_STATE: RefCell<WorkerState> = RefCell::default();
}

#[derive(Default)]
struct WorkerState {
    keys: HashMap<KeyId, secure_enclave_rs::SecureEnclaveKey>,
    next_key_id: KeyId,
}

impl WorkerState {
    fn insert(&mut self, key: secure_enclave_rs::SecureEnclaveKey) -> KeyId {
        let key_id = self.next_key_id;
        self.next_key_id += 1;
        self.keys.insert(key_id, key);
        key_id
    }

    fn key(&self, key_id: KeyId) -> Result<&secure_enclave_rs::SecureEnclaveKey> {
        self.keys
            .get(&key_id)
            .ok_or_else(|| SecureEnclaveError::Other("key is released".to_owned()))
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

    async fn create_key(&self, alias: String) -> Result<Self::Key> {
        let id = run_on_worker(&self.pool, move |state| {
            let key = secure_enclave_rs::SecureEnclaveKey::generate(
                &secure_enclave_rs::SecureEnclaveKeyOptions {
                    tag: alias.as_bytes(),
                    access_flags: secure_enclave_rs::SecAccessControlFlags::USER_PRESENCE,
                    permanent: true,
                },
            )?;
            Ok(state.insert(key))
        })
        .await?;

        Ok(OSSecureEnclaveKey::new(id, self.pool.clone()))
    }

    async fn delete_key(&self, alias: String) -> Result<()> {
        run_on_worker(&self.pool, move |_state| {
            Ok(secure_enclave_rs::SecureEnclaveKey::remove_by_tag(
                alias.as_bytes(),
            )?)
        })
        .await
    }

    async fn get_key(&self, alias: String) -> Result<Self::Key> {
        let id = run_on_worker(&self.pool, move |state| {
            let key = secure_enclave_rs::SecureEnclaveKey::get(alias.as_bytes())?;
            Ok(state.insert(key))
        })
        .await?;

        Ok(OSSecureEnclaveKey::new(id, self.pool.clone()))
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
    async fn decrypt(&self, bytes: Vec<u8>) -> Result<Vec<u8>> {
        let key_id = self.handle.id;
        let mut plaintext = run_on_worker(&self.handle.pool, move |state| {
            Ok(Zeroizing::new(state.key(key_id)?.decrypt(&bytes)?))
        })
        .await?;

        Ok(std::mem::take(&mut *plaintext))
    }

    async fn encrypt(&self, bytes: Vec<u8>) -> Result<Vec<u8>> {
        let key_id = self.handle.id;
        let bytes = Zeroizing::new(bytes);
        run_on_worker(&self.handle.pool, move |state| {
            Ok(state.key(key_id)?.encrypt(&bytes)?)
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

impl From<secure_enclave_rs::Error> for SecureEnclaveError {
    fn from(error: secure_enclave_rs::Error) -> Self {
        match error {
            secure_enclave_rs::Error::AuthFailed => Self::AuthenticationFailed,
            secure_enclave_rs::Error::NotFound => Self::KeyNotFound,
            other => Self::Other(other.to_string()),
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
        .map_err(|_| SecureEnclaveError::Other("secure enclave worker failed".to_owned()))?
}
