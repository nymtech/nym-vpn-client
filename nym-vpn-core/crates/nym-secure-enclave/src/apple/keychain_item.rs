// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::ptr::NonNull;

use objc2_core_foundation::{
    CFBoolean, CFData, CFDictionary, CFError, CFRetained, CFString, CFType, kCFAllocatorNull,
};
use objc2_local_authentication::LAContext;
use objc2_security::{
    SecAccessControl, SecAccessControlCreateFlags, SecItemAdd, SecItemCopyMatching, SecItemDelete,
    SecItemUpdate, errSecDuplicateItem, errSecInteractionNotAllowed, errSecItemNotFound,
    errSecSuccess, kSecAttrAccessControl, kSecAttrAccessGroup, kSecAttrAccessible,
    kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly, kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
    kSecAttrAccount, kSecAttrService, kSecClass, kSecClassGenericPassword, kSecMatchLimit,
    kSecMatchLimitOne, kSecReturnAttributes, kSecReturnData, kSecUseAuthenticationContext,
    kSecUseDataProtectionKeychain, kSecValueData,
};
use zeroize::Zeroizing;

use super::{
    LocalAuthentication,
    local_authentication::non_interactive_context,
    security::{OsStatus, SecError, as_cf_type, dict},
};

#[derive(Debug, Clone, thiserror::Error)]
pub enum KeychainError {
    #[error("failed to {ctx} keychain item: {message}")]
    Security {
        ctx: &'static str,
        /// `OSStatus` of the failure, if any.
        status: Option<OsStatus>,
        message: String,
    },

    #[error("unexpected keychain item data")]
    UnexpectedItemData,

    /// The item requires user presence, but local authentication is locked or the device is
    /// locked.
    #[error("keychain item is locked")]
    Locked,

    #[error("local authentication worker panic")]
    WorkerPanic,
}

pub type KeychainResult<T, E = KeychainError> = std::result::Result<T, E>;

impl KeychainError {
    fn security(ctx: &'static str, err: SecError) -> Self {
        Self::Security {
            ctx,
            status: err.os_status(),
            message: err.to_string(),
        }
    }
}

/// Access to a keychain item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeychainAccess {
    /// Accessible after the first unlock, so the tunnel can start while the device is locked.
    AfterFirstUnlock,
    /// Readable only with the authentication obtained by [`LocalAuthentication::unlock`], and
    /// only while the device is unlocked.
    UserPresence,
}

/// Generic password item in the data protection keychain. It is never synced or included in
/// backups.
#[derive(Clone)]
pub struct KeychainItem {
    service: CFRetained<CFString>,
    account: CFRetained<CFString>,
    access_group: Option<CFRetained<CFString>>,
    access: KeychainAccess,
    auth: LocalAuthentication,
}

// SAFETY: the CoreFoundation strings are immutable.
unsafe impl Send for KeychainItem {}
unsafe impl Sync for KeychainItem {}

impl KeychainItem {
    /// Returns `None` when the item does not exist.
    ///
    /// Fails with [`KeychainError::Locked`] instead of prompting the user when the item requires
    /// [`KeychainAccess::UserPresence`] and is not unlocked.
    pub async fn get_value(&self) -> KeychainResult<Option<Zeroizing<Vec<u8>>>> {
        self.with_context(|item, context| item.get_value_with_context(context))
            .await
    }

    /// Stores `value`, replacing the existing one.
    ///
    /// Fails with [`KeychainError::Locked`] when replacing the value of an item requiring
    /// [`KeychainAccess::UserPresence`] that is not unlocked.
    pub async fn set_value(&self, value: &[u8]) -> KeychainResult<()> {
        let value = Zeroizing::new(value.to_vec());
        self.with_context(move |item, context| item.set_value_with_context(&value, context))
            .await
    }

    /// Stores `value` unless the item exists. Returns whether it was added.
    pub fn add(&self, value: &[u8]) -> KeychainResult<bool> {
        let access = self.access.attribute()?;
        let value = cf_data_no_copy(value)?;
        self.add_item(&[(unsafe { kSecValueData }, &*value), access.pair()])
    }

    /// Whether the item exists, without reading its value or requiring user presence.
    pub fn exists(&self) -> KeychainResult<bool> {
        let context = non_interactive_context();
        let query = unsafe {
            self.query(&[
                (kSecReturnAttributes, CFBoolean::new(true)),
                (kSecMatchLimit, kSecMatchLimitOne),
                (kSecUseAuthenticationContext, as_cf_type(&context)),
            ])
        };

        let mut result: *const CFType = std::ptr::null();
        let status = unsafe { SecItemCopyMatching(query.as_opaque(), &mut result) };
        if let Some(result) = NonNull::new(result.cast_mut()) {
            drop(unsafe { CFRetained::from_raw(result) });
        }
        if status == errSecItemNotFound {
            Ok(false)
        } else {
            check_status(status, "find").map(|()| true)
        }
    }

    /// Deletes the item. Succeeds when it does not exist.
    pub fn delete(&self) -> KeychainResult<()> {
        let query = self.query(&[]);
        let status = unsafe { SecItemDelete(query.as_opaque()) };
        if status == errSecItemNotFound {
            Ok(())
        } else {
            check_status(status, "delete")
        }
    }

    /// Creates the item identified by keychain `service` and `account`. Without `access_group`,
    /// the item goes to the default access group of the process.
    pub(crate) fn new(
        service: &str,
        account: &str,
        access_group: Option<&str>,
        access: KeychainAccess,
        auth: LocalAuthentication,
    ) -> Self {
        KeychainItem {
            service: CFString::from_str(service),
            account: CFString::from_str(account),
            access_group: access_group.map(CFString::from_str),
            access,
            auth,
        }
    }

    /// Runs `f` with the context satisfying the access control of the item. Items requiring user
    /// presence are accessed on the worker thread with the shared authentication.
    async fn with_context<T, F>(&self, f: F) -> KeychainResult<T>
    where
        F: FnOnce(&Self, &LAContext) -> KeychainResult<T> + Send + 'static,
        T: Send + 'static,
    {
        match self.access {
            KeychainAccess::AfterFirstUnlock => f(self, &non_interactive_context()),
            KeychainAccess::UserPresence => {
                let item = self.clone();
                let result = self
                    .auth
                    .run(move |auth| f(&item, &auth.context))
                    .await
                    .map_err(|_| KeychainError::WorkerPanic)?;

                match result {
                    Err(KeychainError::Security {
                        status: Some(status),
                        ..
                    }) if status == errSecInteractionNotAllowed => Err(KeychainError::Locked),
                    result => result,
                }
            }
        }
    }

    fn get_value_with_context(
        &self,
        context: &LAContext,
    ) -> KeychainResult<Option<Zeroizing<Vec<u8>>>> {
        let query = unsafe {
            self.query(&[
                (kSecReturnData, CFBoolean::new(true)),
                (kSecMatchLimit, kSecMatchLimitOne),
                (kSecUseAuthenticationContext, as_cf_type(context)),
            ])
        };

        let mut result: *const CFType = std::ptr::null();
        let status = unsafe { SecItemCopyMatching(query.as_opaque(), &mut result) };
        if status == errSecItemNotFound {
            return Ok(None);
        }
        check_status(status, "load")?;

        let result = NonNull::new(result.cast_mut()).ok_or(KeychainError::UnexpectedItemData)?;
        // The returned CFData is immutable and cannot be zeroized.
        let data = unsafe { CFRetained::from_raw(result) }
            .downcast::<CFData>()
            .map_err(|_| KeychainError::UnexpectedItemData)?;

        Ok(Some(Zeroizing::new(data.to_vec())))
    }

    fn set_value_with_context(&self, value: &[u8], context: &LAContext) -> KeychainResult<()> {
        let access = self.access.attribute()?;
        let value = cf_data_no_copy(value)?;
        let attributes = dict(&[(unsafe { kSecValueData }, &*value), access.pair()]);
        let query = unsafe { self.query(&[(kSecUseAuthenticationContext, as_cf_type(context))]) };

        let status = unsafe { SecItemUpdate(query.as_opaque(), attributes.as_opaque()) };
        if status != errSecItemNotFound {
            return check_status(status, "update");
        }

        match self.add_item(&[(unsafe { kSecValueData }, &*value), access.pair()])? {
            true => Ok(()),
            // Added by another process since the update above.
            false => {
                let status = unsafe { SecItemUpdate(query.as_opaque(), attributes.as_opaque()) };
                check_status(status, "update")
            }
        }
    }

    fn add_item(&self, attributes: &[(&CFString, &CFType)]) -> KeychainResult<bool> {
        let item = self.query(attributes);
        let status = unsafe { SecItemAdd(item.as_opaque(), std::ptr::null_mut()) };
        if status == errSecDuplicateItem {
            Ok(false)
        } else {
            check_status(status, "add").map(|()| true)
        }
    }

    /// Query identifying the item, extended with `extra` pairs.
    fn query(&self, extra: &[(&CFString, &CFType)]) -> CFRetained<CFDictionary<CFString, CFType>> {
        let mut pairs: Vec<(&CFString, &CFType)> = unsafe {
            vec![
                (kSecClass, kSecClassGenericPassword),
                (kSecAttrService, &self.service),
                (kSecAttrAccount, &self.account),
                (kSecUseDataProtectionKeychain, CFBoolean::new(true)),
            ]
        };
        if let Some(access_group) = &self.access_group {
            pairs.push((unsafe { kSecAttrAccessGroup }, access_group));
        }
        pairs.extend_from_slice(extra);
        dict(&pairs)
    }
}

impl KeychainAccess {
    fn attribute(self) -> KeychainResult<AccessAttribute> {
        match self {
            Self::AfterFirstUnlock => Ok(AccessAttribute::Accessible(unsafe {
                kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
            })),
            Self::UserPresence => {
                let mut err: *mut CFError = std::ptr::null_mut();
                let acl = unsafe {
                    SecAccessControl::with_flags(
                        None,
                        kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
                        SecAccessControlCreateFlags::UserPresence,
                        &mut err,
                    )
                };
                if let Some(err) = NonNull::new(err) {
                    let err = SecError::Cf(unsafe { CFRetained::from_raw(err) });
                    return Err(KeychainError::security("create access control for", err));
                }
                acl.map(AccessAttribute::AccessControl)
                    .ok_or(KeychainError::UnexpectedItemData)
            }
        }
    }
}

enum AccessAttribute {
    Accessible(&'static CFType),
    AccessControl(CFRetained<SecAccessControl>),
}

impl AccessAttribute {
    fn pair(&self) -> (&'static CFString, &CFType) {
        match self {
            Self::Accessible(accessible) => (unsafe { kSecAttrAccessible }, accessible),
            Self::AccessControl(acl) => (unsafe { kSecAttrAccessControl }, acl),
        }
    }
}

fn cf_data_no_copy(value: &[u8]) -> KeychainResult<CFRetained<CFData>> {
    unsafe { CFData::with_bytes_no_copy(None, value.as_ptr(), value.len() as _, kCFAllocatorNull) }
        .ok_or(KeychainError::UnexpectedItemData)
}

fn check_status(status: OsStatus, ctx: &'static str) -> KeychainResult<()> {
    if status == errSecSuccess {
        Ok(())
    } else {
        Err(KeychainError::security(ctx, SecError::Os(status)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Keychain;

    // The data protection keychain requires a signed binary with keychain entitlements.
    #[tokio::test]
    #[ignore = "requires keychain entitlements"]
    async fn sets_adds_and_deletes() {
        let item = Keychain::new(LocalAuthentication::new(), None).item(
            "net.nymtech.vpn.tests.keychain-item",
            "test",
            KeychainAccess::AfterFirstUnlock,
        );

        item.delete().unwrap();
        assert!(!item.exists().unwrap());
        assert!(item.get_value().await.unwrap().is_none());

        assert!(item.add(b"first").unwrap());
        assert!(!item.add(b"second").unwrap());
        assert!(item.exists().unwrap());
        assert_eq!(
            item.get_value().await.unwrap().unwrap().as_slice(),
            b"first"
        );

        item.set_value(b"third").await.unwrap();
        assert_eq!(
            item.get_value().await.unwrap().unwrap().as_slice(),
            b"third"
        );

        item.delete().unwrap();
        assert!(item.get_value().await.unwrap().is_none());
    }
}
