// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use windows::{
    Win32::{
        Foundation::{NTE_BAD_KEYSET, NTE_EXISTS, NTE_NOT_FOUND},
        Security::Cryptography::{
            BCRYPT_OAEP_PADDING_INFO, BCRYPT_RSA_ALGORITHM, BCRYPT_SHA256_ALGORITHM, CERT_KEY_SPEC,
            MS_PLATFORM_CRYPTO_PROVIDER, NCRYPT_FLAGS, NCRYPT_KEY_HANDLE, NCRYPT_LENGTH_PROPERTY,
            NCRYPT_PAD_OAEP_FLAG, NCRYPT_PROV_HANDLE, NCRYPT_SILENT_FLAG, NCryptCreatePersistedKey,
            NCryptDecrypt, NCryptDeleteKey, NCryptEncrypt, NCryptFinalizeKey, NCryptFreeObject,
            NCryptOpenKey, NCryptOpenStorageProvider, NCryptSetProperty,
        },
    },
    core::HSTRING,
};
use zeroize::Zeroizing;

use crate::{Result, SecureEnclaveError};

const RSA_KEY_BITS: u32 = 2048;
const AES_KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

/// Platform Crypto Provider, storing keys in the TPM.
pub struct Provider(NCRYPT_PROV_HANDLE);

impl Provider {
    /// Fails when the machine has no usable TPM.
    pub fn open() -> windows::core::Result<Self> {
        let mut handle = NCRYPT_PROV_HANDLE::default();
        unsafe { NCryptOpenStorageProvider(&mut handle, MS_PLATFORM_CRYPTO_PROVIDER, 0) }?;
        Ok(Self(handle))
    }

    pub fn open_key(self, name: &str) -> Result<Key> {
        let mut handle = NCRYPT_KEY_HANDLE::default();
        unsafe {
            NCryptOpenKey(
                self.0,
                &mut handle,
                &HSTRING::from(name),
                CERT_KEY_SPEC(0),
                NCRYPT_SILENT_FLAG,
            )
        }
        .map_err(|err| {
            if err.code() == NTE_BAD_KEYSET || err.code() == NTE_NOT_FOUND {
                SecureEnclaveError::KeyNotFound
            } else {
                other_error("open TPM key", err)
            }
        })?;
        Ok(Key {
            handle,
            _provider: self,
        })
    }

    /// Creates a non-exportable RSA key for wrapping the keys encrypting the data.
    pub fn create_key(self, name: &str) -> Result<Key> {
        let mut handle = NCRYPT_KEY_HANDLE::default();
        unsafe {
            NCryptCreatePersistedKey(
                self.0,
                &mut handle,
                BCRYPT_RSA_ALGORITHM,
                &HSTRING::from(name),
                CERT_KEY_SPEC(0),
                NCRYPT_FLAGS(0),
            )
        }
        .map_err(|err| map_create_error("create TPM key", err))?;
        let key = Key {
            handle,
            _provider: self,
        };

        unsafe {
            NCryptSetProperty(
                key.handle.into(),
                NCRYPT_LENGTH_PROPERTY,
                &RSA_KEY_BITS.to_le_bytes(),
                NCRYPT_FLAGS(0),
            )
        }
        .map_err(|err| other_error("set TPM key length", err))?;

        unsafe { NCryptFinalizeKey(key.handle, NCRYPT_SILENT_FLAG) }
            .map_err(|err| map_create_error("finalize TPM key", err))?;

        Ok(key)
    }
}

impl Drop for Provider {
    fn drop(&mut self) {
        let _ = unsafe { NCryptFreeObject(self.0.into()) };
    }
}

/// Key in the TPM, keeping its provider open.
pub struct Key {
    handle: NCRYPT_KEY_HANDLE,
    _provider: Provider,
}

impl Key {
    pub fn delete(mut self) -> Result<()> {
        unsafe { NCryptDeleteKey(self.handle, 0) }
            .map_err(|err| other_error("delete TPM key", err))?;
        // A successful deletion frees the handle.
        self.handle = NCRYPT_KEY_HANDLE::default();
        Ok(())
    }

    /// Encrypts the data with a random AES-256-GCM key wrapped with the TPM key.
    ///
    /// The output is the length of the wrapped key as u16 LE, the wrapped key, the nonce and the
    /// ciphertext.
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let aes_key = Zeroizing::new(rand::random::<[u8; AES_KEY_LEN]>());
        let nonce: [u8; NONCE_LEN] = rand::random();

        let ciphertext = Aes256Gcm::new((&*aes_key).into())
            .encrypt(&Nonce::from(nonce), data)
            .map_err(|_| SecureEnclaveError::Other("failed to encrypt data".to_owned()))?;
        let wrapped_key = self.rsa_encrypt(&*aes_key)?;
        let wrapped_key_len = u16::try_from(wrapped_key.len())
            .map_err(|_| SecureEnclaveError::Other("wrapped key is too long".to_owned()))?;

        let mut output = Vec::with_capacity(2 + wrapped_key.len() + NONCE_LEN + ciphertext.len());
        output.extend_from_slice(&wrapped_key_len.to_le_bytes());
        output.extend_from_slice(&wrapped_key);
        output.extend_from_slice(&nonce);
        output.extend_from_slice(&ciphertext);
        Ok(output)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let (wrapped_key_len, data) = data.split_first_chunk::<2>().ok_or_else(invalid_data)?;
        let wrapped_key_len = usize::from(u16::from_le_bytes(*wrapped_key_len));
        let (wrapped_key, data) = data
            .split_at_checked(wrapped_key_len)
            .ok_or_else(invalid_data)?;
        let (nonce, ciphertext) = data
            .split_first_chunk::<NONCE_LEN>()
            .ok_or_else(invalid_data)?;

        let aes_key = self.rsa_decrypt(wrapped_key)?;
        let aes_key: &[u8; AES_KEY_LEN] =
            aes_key.as_slice().try_into().map_err(|_| invalid_data())?;

        Aes256Gcm::new(aes_key.into())
            .decrypt(&Nonce::from(*nonce), ciphertext)
            .map_err(|_| SecureEnclaveError::Other("failed to decrypt data".to_owned()))
    }

    fn rsa_encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let padding = oaep_padding();
        let padding = Some(&raw const padding as *const core::ffi::c_void);
        let flags = NCRYPT_PAD_OAEP_FLAG | NCRYPT_SILENT_FLAG;

        let mut len = 0;
        unsafe { NCryptEncrypt(self.handle, Some(data), padding, None, &mut len, flags) }
            .map_err(|err| other_error("get TPM encryption size", err))?;

        let mut output = vec![0; len as usize];
        unsafe {
            NCryptEncrypt(
                self.handle,
                Some(data),
                padding,
                Some(&mut output),
                &mut len,
                flags,
            )
        }
        .map_err(|err| other_error("encrypt with TPM key", err))?;
        output.truncate(len as usize);
        Ok(output)
    }

    fn rsa_decrypt(&self, data: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
        let padding = oaep_padding();
        let padding = Some(&raw const padding as *const core::ffi::c_void);
        let flags = NCRYPT_PAD_OAEP_FLAG | NCRYPT_SILENT_FLAG;

        let mut len = 0;
        unsafe { NCryptDecrypt(self.handle, Some(data), padding, None, &mut len, flags) }
            .map_err(|err| other_error("get TPM decryption size", err))?;

        let mut output = Zeroizing::new(vec![0; len as usize]);
        unsafe {
            NCryptDecrypt(
                self.handle,
                Some(data),
                padding,
                Some(&mut output),
                &mut len,
                flags,
            )
        }
        .map_err(|err| other_error("decrypt with TPM key", err))?;
        output.truncate(len as usize);
        Ok(output)
    }
}

impl Drop for Key {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            let _ = unsafe { NCryptFreeObject(self.handle.into()) };
        }
    }
}

fn oaep_padding() -> BCRYPT_OAEP_PADDING_INFO {
    BCRYPT_OAEP_PADDING_INFO {
        pszAlgId: BCRYPT_SHA256_ALGORITHM,
        pbLabel: std::ptr::null_mut(),
        cbLabel: 0,
    }
}

fn map_create_error(ctx: &str, err: windows::core::Error) -> SecureEnclaveError {
    if err.code() == NTE_EXISTS {
        SecureEnclaveError::KeyAlreadyExists
    } else {
        other_error(ctx, err)
    }
}

fn other_error(ctx: &str, err: windows::core::Error) -> SecureEnclaveError {
    SecureEnclaveError::Other(format!("failed to {ctx}: {err}"))
}

fn invalid_data() -> SecureEnclaveError {
    SecureEnclaveError::Other("invalid encrypted data".to_owned())
}
