// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod auth;
mod auth_error;
mod keychain;
mod keychain_item;
mod local_authentication;
mod security;
mod zeroing_allocator;

use std::{cell::RefCell, collections::HashMap, sync::Arc};

use objc2_core_foundation::{CFRetained, CFString};
use objc2_security::{
    SecKey, errSecAuthFailed, errSecDataNotAvailable, errSecItemNotFound, errSecUserCanceled,
};

use zeroize::Zeroizing;

use crate::{Result, SecureEnclaveError, SecureEnclaveKey, SecureEnclaveProvider};
use auth::AuthState;
use auth_error::AuthErrorKind;
use security::{SecError, create_sec_key, decrypt_data, delete_sec_key, encrypt_data, get_sec_key};

pub use auth::LocalAuthentication;
pub use keychain::Keychain;
pub use keychain_item::{KeychainAccess, KeychainError, KeychainItem, KeychainResult};

type KeyId = u64;

thread_local! {
    /// Keys owned by the worker thread. `SecKey` cannot be sent
    /// between threads, so it never leaves the thread it was created on.
    static KEYS: RefCell<Keys> = RefCell::default();
}

#[derive(Default)]
struct Keys {
    entries: HashMap<KeyId, KeyEntry>,
    next_key_id: KeyId,
}

struct KeyEntry {
    tag: Vec<u8>,
    access_group: Option<String>,
    key: CFRetained<SecKey>,
    /// Generation of the authentication context the key was obtained with.
    auth_generation: u64,
}

impl Keys {
    fn insert(
        &mut self,
        auth: &AuthState,
        tag: Vec<u8>,
        access_group: Option<String>,
        key: CFRetained<SecKey>,
    ) -> KeyId {
        let key_id = self.next_key_id;
        self.next_key_id += 1;
        self.entries.insert(
            key_id,
            KeyEntry {
                tag,
                access_group,
                key,
                auth_generation: auth.generation,
            },
        );
        key_id
    }

    /// Returns the key, fetching it again if the authentication context changed since the key
    /// was obtained.
    fn key(&mut self, auth: &AuthState, key_id: KeyId) -> Result<&CFRetained<SecKey>> {
        let entry = self
            .entries
            .get_mut(&key_id)
            .ok_or_else(|| SecureEnclaveError::Other("key is released".to_owned()))?;

        if entry.auth_generation != auth.generation {
            let access_group = entry.access_group.as_deref().map(CFString::from_str);
            entry.key = get_sec_key(&entry.tag, access_group.as_deref(), &auth.context)?;
            entry.auth_generation = auth.generation;
        }

        Ok(&entry.key)
    }
}

pub struct AppleSecureEnclaveProvider {
    auth: LocalAuthentication,
    /// Keychain access group of the keys, or the default access group of the process if `None`.
    access_group: Option<String>,
}

impl AppleSecureEnclaveProvider {
    pub fn new(auth: LocalAuthentication, access_group: Option<&str>) -> Self {
        Self {
            auth,
            access_group: access_group.map(ToOwned::to_owned),
        }
    }

    /// Run the closure on the worker thread with the authentication state and the keys.
    async fn run<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&AuthState, &mut Keys) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        self.auth
            .run(move |auth| KEYS.with_borrow_mut(|keys| f(auth, keys)))
            .await?
    }
}

#[async_trait::async_trait]
impl SecureEnclaveProvider for AppleSecureEnclaveProvider {
    type Key = OSSecureEnclaveKey;

    async fn create_key(&self, alias: &str, user_presence: bool) -> Result<Self::Key> {
        let alias_bytes = alias.as_bytes().to_vec();
        let access_group = self.access_group.clone();
        let id = self
            .run(move |auth, keys| {
                let cf_access_group = access_group.as_deref().map(CFString::from_str);
                let cf_access_group = cf_access_group.as_deref();
                // Prevent creating more than one key with the same alias.
                match get_sec_key(&alias_bytes, cf_access_group, &auth.context)
                    .map_err(SecureEnclaveError::from)
                {
                    Ok(_) => Err(SecureEnclaveError::KeyAlreadyExists),
                    Err(SecureEnclaveError::KeyNotFound) => {
                        let key = create_sec_key(
                            &alias_bytes,
                            cf_access_group,
                            user_presence,
                            &auth.context,
                        )?;
                        Ok(keys.insert(auth, alias_bytes, access_group, key))
                    }
                    Err(err) => Err(err),
                }
            })
            .await?;
        Ok(OSSecureEnclaveKey::new(id, self.auth.clone()))
    }

    async fn delete_key(&self, alias: &str) -> Result<()> {
        let alias_bytes = alias.as_bytes().to_vec();
        let access_group = self.access_group.clone();
        self.run(move |_auth, _keys| {
            let access_group = access_group.as_deref().map(CFString::from_str);
            Ok(delete_sec_key(&alias_bytes, access_group.as_deref())?)
        })
        .await
    }

    async fn get_key(&self, alias: &str) -> Result<Self::Key> {
        let alias_bytes = alias.as_bytes().to_vec();
        let access_group = self.access_group.clone();
        let id = self
            .run(move |auth, keys| {
                let cf_access_group = access_group.as_deref().map(CFString::from_str);
                let key = get_sec_key(&alias_bytes, cf_access_group.as_deref(), &auth.context)?;
                Ok(keys.insert(auth, alias_bytes, access_group, key))
            })
            .await?;
        Ok(OSSecureEnclaveKey::new(id, self.auth.clone()))
    }

    async fn unlock(&self, reason: &str) -> Result<()> {
        self.auth.unlock(reason).await
    }

    async fn lock(&self) -> Result<()> {
        self.auth.lock().await
    }
}

#[derive(Clone)]
pub struct OSSecureEnclaveKey {
    handle: Arc<KeyHandle>,
}

impl OSSecureEnclaveKey {
    fn new(id: KeyId, auth: LocalAuthentication) -> Self {
        Self {
            handle: Arc::new(KeyHandle { id, auth }),
        }
    }

    async fn with_key<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&SecKey) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let key_id = self.handle.id;
        self.handle
            .auth
            .run(move |auth| KEYS.with_borrow_mut(|keys| f(keys.key(auth, key_id)?)))
            .await?
    }
}

#[async_trait::async_trait]
impl SecureEnclaveKey for OSSecureEnclaveKey {
    async fn decrypt(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let bytes = bytes.to_vec();
        self.with_key(move |key| Ok(decrypt_data(key, &bytes)?))
            .await
    }

    async fn encrypt(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let bytes = Zeroizing::new(bytes.to_vec());
        self.with_key(move |key| Ok(encrypt_data(key, &bytes)?))
            .await
    }
}

/// Reference to the key owned by the worker thread. The key is released once the last handle is
/// dropped.
struct KeyHandle {
    id: KeyId,
    auth: LocalAuthentication,
}

impl Drop for KeyHandle {
    fn drop(&mut self) {
        let key_id = self.id;
        self.auth.spawn(move |_auth| {
            KEYS.with_borrow_mut(|keys| keys.entries.remove(&key_id));
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
