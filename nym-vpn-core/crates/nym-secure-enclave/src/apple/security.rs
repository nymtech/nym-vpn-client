// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::ptr::NonNull;

use objc2_core_foundation::{
    CFBoolean, CFData, CFDictionary, CFError, CFIndex, CFNumber, CFRetained, CFString, CFType,
    kCFAllocatorNull,
};
use objc2_security::{
    SecAccessControl, SecAccessControlCreateFlags, SecCopyErrorMessageString, SecItemCopyMatching,
    SecItemDelete, SecKey, SecKeyAlgorithm, errSecSuccess, kSecAttrAccessControl,
    kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly, kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
    kSecAttrApplicationTag, kSecAttrIsPermanent, kSecAttrKeySizeInBits, kSecAttrKeyType,
    kSecAttrKeyTypeECSECPrimeRandom, kSecAttrTokenID, kSecAttrTokenIDSecureEnclave, kSecClass,
    kSecClassKey, kSecKeyAlgorithmECIESEncryptionStandardVariableIVX963SHA256AESGCM,
    kSecMatchLimit, kSecMatchLimitOne, kSecPrivateKeyAttrs, kSecReturnRef,
};

const KEY_SIZE: i32 = 256;

pub(super) type OsStatus = i32;

#[derive(Debug)]
pub(super) enum SecError {
    Os(OsStatus),
    Cf(CFRetained<CFError>),
    Internal(&'static str),
}

impl From<OsStatus> for SecError {
    fn from(value: OsStatus) -> Self {
        Self::Os(value)
    }
}

impl std::fmt::Display for SecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Os(code) => {
                let res = unsafe { SecCopyErrorMessageString(*code, std::ptr::null_mut()) };

                match res {
                    Some(msg) => f.write_str(&msg.to_string()),
                    None => write!(f, "unknown osstatus: {code}"),
                }
            }
            Self::Cf(err) => f.write_str(&err.to_string()),
            Self::Internal(msg) => f.write_str(msg),
        }
    }
}

pub(super) type SecResult<T, E = SecError> = std::result::Result<T, E>;

pub(super) fn get_sec_key(tag: &[u8]) -> SecResult<CFRetained<SecKey>> {
    let query = unsafe {
        dict(&[
            (kSecClass, kSecClassKey),
            (kSecAttrKeyType, kSecAttrKeyTypeECSECPrimeRandom),
            (kSecAttrApplicationTag, &CFData::from_bytes(tag)),
            (kSecReturnRef, CFBoolean::new(true)),
            (kSecMatchLimit, kSecMatchLimitOne),
        ])
    };

    let mut result: *const CFType = std::ptr::null();
    let status = unsafe { SecItemCopyMatching(query.as_opaque(), &mut result) };
    wrap_os_status(status)?;

    let result = NonNull::new(result.cast_mut()).ok_or(SecError::Internal(
        "unexpected null returned from SecItemCopyMatching()",
    ))?;
    let key_ref = unsafe { CFRetained::from_raw(result) }
        .downcast::<SecKey>()
        .map_err(|_| {
            SecError::Internal("failed to downcast return value of SecItemCopyMatching() to SecKey")
        })?;

    Ok(key_ref)
}

pub(super) fn delete_sec_key(tag: &[u8]) -> SecResult<()> {
    let tag = CFData::from_bytes(tag);

    let query = unsafe {
        dict(&[
            (kSecClass, kSecClassKey),
            (kSecAttrKeyType, kSecAttrKeyTypeECSECPrimeRandom),
            (kSecAttrApplicationTag, &tag),
        ])
    };

    let status = unsafe { SecItemDelete(query.as_opaque()) };
    wrap_os_status(status)
}

pub(super) fn create_sec_key(tag: &[u8], user_presence: bool) -> SecResult<CFRetained<SecKey>> {
    let mut flags = SecAccessControlCreateFlags::PrivateKeyUsage;
    if user_presence {
        flags |= SecAccessControlCreateFlags::UserPresence;
    }

    let protection = unsafe {
        if user_presence {
            kSecAttrAccessibleWhenUnlockedThisDeviceOnly
        } else {
            kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
        }
    };

    let mut err: *mut CFError = std::ptr::null_mut();
    let acl = unsafe { SecAccessControl::with_flags(None, protection, flags, &mut err) };
    unsafe { wrap_cf_error(err)? };
    let acl = acl.ok_or(SecError::Internal(
        "unexpected null returned from SecAccessControl::with_flags()",
    ))?;

    let tag = CFData::from_bytes(tag);
    let key_size = CFNumber::new_i32(KEY_SIZE);
    let private_attrs = unsafe {
        dict(&[
            (kSecAttrIsPermanent, CFBoolean::new(true)),
            (kSecAttrApplicationTag, &tag),
            (kSecAttrAccessControl, &acl),
        ])
    };

    let params = unsafe {
        dict(&[
            (kSecAttrKeyType, kSecAttrKeyTypeECSECPrimeRandom),
            (kSecAttrKeySizeInBits, &key_size),
            (kSecAttrTokenID, kSecAttrTokenIDSecureEnclave),
            (kSecPrivateKeyAttrs, private_attrs.as_opaque()),
        ])
    };

    let mut err: *mut CFError = std::ptr::null_mut();
    let key = unsafe { SecKey::new_random_key(params.as_opaque(), &mut err) };
    unsafe { wrap_cf_error(err)? };

    key.ok_or(SecError::Internal(
        "unexpected null returned from SecKey::new_random_key()",
    ))
}

pub(super) fn decrypt_data(private_key: &SecKey, cipher_text: &[u8]) -> SecResult<Vec<u8>> {
    let cipher_text = unsafe {
        CFData::with_bytes_no_copy(
            None,
            cipher_text.as_ptr(),
            cipher_text.len() as CFIndex,
            kCFAllocatorNull,
        )
    }
    .ok_or(SecError::Internal(
        "failed to allocate CFData::with_bytes_no_copy",
    ))?;

    let mut err: *mut CFError = std::ptr::null_mut();
    let plain_text = unsafe { private_key.decrypted_data(key_algorithm(), &cipher_text, &mut err) };
    unsafe { wrap_cf_error(err)? };

    // WARN: it does not seem there is a way to zero out returned CFData as it's immutable.
    let data = plain_text.ok_or(SecError::Internal(
        "unexpected null returned from SecKey::decrypted_data()",
    ))?;
    let plain_text = data.to_vec();

    Ok(plain_text)
}

pub(super) fn encrypt_data(private_key: &SecKey, plain_text: &[u8]) -> SecResult<Vec<u8>> {
    let public_key = unsafe { private_key.public_key() }
        .ok_or(SecError::Internal("failed to obtain public key"))?;

    let plain_text = unsafe {
        CFData::with_bytes_no_copy(
            None,
            plain_text.as_ptr(),
            plain_text.len() as CFIndex,
            kCFAllocatorNull,
        )
    }
    .ok_or(SecError::Internal(
        "failed to allocate CFData::with_bytes_no_copy",
    ))?;

    let mut err: *mut CFError = std::ptr::null_mut();
    let cipher_data = unsafe { public_key.encrypted_data(key_algorithm(), &plain_text, &mut err) };
    unsafe { wrap_cf_error(err)? };

    let data = cipher_data.ok_or(SecError::Internal(
        "unexpected null returned from SecKey::encrypted_data()",
    ))?;

    Ok(data.to_vec())
}

fn key_algorithm() -> &'static SecKeyAlgorithm {
    unsafe { kSecKeyAlgorithmECIESEncryptionStandardVariableIVX963SHA256AESGCM }
}

unsafe fn wrap_cf_error(err: *mut CFError) -> SecResult<()> {
    match NonNull::new(err).map(|err| unsafe { CFRetained::from_raw(err) }) {
        Some(err) => Err(SecError::Cf(err)),
        None => Ok(()),
    }
}

fn wrap_os_status(status: OsStatus) -> SecResult<()> {
    if status == errSecSuccess {
        Ok(())
    } else {
        Err(SecError::from(status))
    }
}

fn dict(pairs: &[(&CFString, &CFType)]) -> CFRetained<CFDictionary<CFString, CFType>> {
    let (keys, vals): (Vec<_>, Vec<_>) = pairs.iter().copied().unzip();
    CFDictionary::from_slices(&keys, &vals)
}
