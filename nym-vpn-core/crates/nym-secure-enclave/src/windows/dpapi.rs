// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::{
    Win32::{
        Foundation::{HLOCAL, LocalFree},
        Security::Cryptography::{
            CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
        },
    },
    core::PCWSTR,
};
use zeroize::Zeroize;

use crate::{Result, SecureEnclaveError};

/// Encrypts the data with DPAPI for the current user. The `entropy` is required for decryption.
pub fn protect(data: &[u8], entropy: &[u8]) -> Result<Vec<u8>> {
    let input = blob(data)?;
    let entropy = blob(entropy)?;
    let mut output = CRYPT_INTEGER_BLOB::default();

    unsafe {
        CryptProtectData(
            &input,
            PCWSTR::null(),
            Some(&entropy),
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    }
    .map_err(|err| SecureEnclaveError::Other(format!("failed to protect data: {err}")))?;

    Ok(take_output(output))
}

pub fn unprotect(data: &[u8], entropy: &[u8]) -> Result<Vec<u8>> {
    let input = blob(data)?;
    let entropy = blob(entropy)?;
    let mut output = CRYPT_INTEGER_BLOB::default();

    unsafe {
        CryptUnprotectData(
            &input,
            None,
            Some(&entropy),
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    }
    .map_err(|err| SecureEnclaveError::Other(format!("failed to unprotect data: {err}")))?;

    Ok(take_output(output))
}

fn blob(data: &[u8]) -> Result<CRYPT_INTEGER_BLOB> {
    let len = u32::try_from(data.len())
        .map_err(|_| SecureEnclaveError::Other("data is too long".to_owned()))?;
    Ok(CRYPT_INTEGER_BLOB {
        cbData: len,
        pbData: data.as_ptr().cast_mut(),
    })
}

/// Copies the output allocated by DPAPI, zeroing and freeing the original buffer.
fn take_output(output: CRYPT_INTEGER_BLOB) -> Vec<u8> {
    if output.pbData.is_null() {
        Vec::new()
    } else {
        let buffer =
            unsafe { std::slice::from_raw_parts_mut(output.pbData, output.cbData as usize) };
        let data = buffer.to_vec();
        buffer.zeroize();
        unsafe { LocalFree(Some(HLOCAL(output.pbData.cast()))) };
        data
    }
}
