// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{MnemonicStorage, MnemonicStorageError, StoredMnemonic};
use std::os::raw::c_void;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::{
    fs::{self, File},
    mem,
    path::PathBuf,
    ptr,
};
#[cfg(windows)]
use widestring::U16CString;
#[cfg(windows)]
use winapi::um::winnt::SECURITY_MAX_SID_SIZE;
#[cfg(windows)]
use windows_sys::Win32::Foundation::*;
#[cfg(windows)]
use windows_sys::Win32::Security::Authorization::{SetNamedSecurityInfoW, SE_FILE_OBJECT};
#[cfg(windows)]
use windows_sys::Win32::Security::*;
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::*;
#[cfg(windows)]
use windows_sys::Win32::System::SystemServices::SECURITY_DESCRIPTOR_REVISION;

#[derive(Debug, thiserror::Error)]
pub enum OnDiskMnemonicStorageError {
    #[error("mnemonic already stored")]
    MnemonicAlreadyStored { path: PathBuf },

    #[error("failed to create file")]
    FileCreateError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to open file")]
    FileOpenError(#[source] std::io::Error),

    #[error("failed to read mnemonic from file")]
    ReadError(#[source] serde_json::Error),

    #[error("failed to write mnemonic to file")]
    WriteError(#[source] serde_json::Error),

    #[error("failed to remove mnemonic file")]
    RemoveError(#[source] std::io::Error),
}

impl MnemonicStorageError for OnDiskMnemonicStorageError {
    fn is_mnemonic_stored(&self) -> bool {
        matches!(
            self,
            OnDiskMnemonicStorageError::MnemonicAlreadyStored { .. }
        )
    }
}

pub struct OnDiskMnemonicStorage {
    path: PathBuf,
}

impl OnDiskMnemonicStorage {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl MnemonicStorage for OnDiskMnemonicStorage {
    type StorageError = OnDiskMnemonicStorageError;

    async fn store_mnemonic(
        &self,
        mnemonic: bip39::Mnemonic,
    ) -> Result<(), OnDiskMnemonicStorageError> {
        let name = "default".to_string();
        let nonce = 0;
        let stored_mnemonic = StoredMnemonic {
            name,
            mnemonic,
            nonce,
        };

        tracing::info!("Storing mnemonic to: {}", self.path.display());

        // Error if the file already exists
        if self.path.exists() {
            return Err(OnDiskMnemonicStorageError::MnemonicAlreadyStored {
                path: self.path.clone(),
            });
        }

        // Create parent directories
        tracing::trace!("Creating parent directories for: {}", self.path.display());
        if let Some(parent) = self.path.parent() {
            tracing::trace!("Creating parent directory: {}", parent.display());
            fs::create_dir_all(parent).map_err(|err| {
                OnDiskMnemonicStorageError::FileCreateError {
                    path: parent.to_path_buf(),
                    source: err,
                }
            })?;

            #[cfg(unix)]
            {
                // Set directory permissions to 700 (rwx------)
                tracing::trace!("Set directory permissions to 700 (rwx------)");
                let permissions = fs::Permissions::from_mode(0o700);
                fs::set_permissions(parent, permissions).map_err(|source| {
                    OnDiskMnemonicStorageError::FileCreateError {
                        path: parent.to_path_buf(),
                        source,
                    }
                })?;
            }

            #[cfg(windows)]
            {
                // Set directory permissions to parent directory on Windows
                tracing::trace!("Setting ACL for parent directory on Windows");
                set_secure_permissions_windows(parent).map_err(|err| {
                    OnDiskMnemonicStorageError::FileCreateError {
                        path: parent.to_path_buf(),
                        source: err,
                    }
                })?;
            }
        }

        // Another layer of defense, only create the file if it doesn't already exist
        tracing::debug!("Only creating the file if it doesn't already exist");
        let file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&self.path)
            .map_err(|err| OnDiskMnemonicStorageError::FileCreateError {
                path: self.path.clone(),
                source: err,
            })?;

        serde_json::to_writer(file, &stored_mnemonic)
            .map_err(OnDiskMnemonicStorageError::WriteError)?;

        #[cfg(unix)]
        {
            // Set directory permissions to 600 (rw------)
            let permissions = fs::Permissions::from_mode(0o600);
            fs::set_permissions(self.path.clone(), permissions).map_err(|source| {
                OnDiskMnemonicStorageError::FileCreateError {
                    path: self.path.clone(),
                    source,
                }
            })?;
        }

        #[cfg(windows)]
        {
            // Setting ACL for file on Windows
            set_secure_permissions_windows(&self.path).map_err(|err| {
                OnDiskMnemonicStorageError::FileCreateError {
                    path: self.path.clone(),
                    source: err,
                }
            })?;
        }

        Ok(())
    }

    async fn load_mnemonic(&self) -> Result<bip39::Mnemonic, OnDiskMnemonicStorageError> {
        tracing::debug!("Opening: {}", self.path.display());

        // Make sure that the file has permissions set to 600 (rw------)
        #[cfg(unix)]
        {
            let permissions = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&self.path, permissions)
                .map_err(OnDiskMnemonicStorageError::FileOpenError)?;
        }

        let file = File::open(&self.path).map_err(OnDiskMnemonicStorageError::FileOpenError)?;
        serde_json::from_reader(file)
            .map_err(OnDiskMnemonicStorageError::ReadError)
            .map(|s: StoredMnemonic| s.mnemonic.clone())
    }

    async fn remove_mnemonic(&self) -> Result<(), OnDiskMnemonicStorageError> {
        if !self.path.exists() {
            return Ok(());
        }
        std::fs::remove_file(&self.path).map_err(OnDiskMnemonicStorageError::RemoveError)
    }
}

#[cfg(windows)]
fn set_secure_permissions_windows<P: AsRef<Path>>(path: P) -> Result<(), std::io::Error> {
    let wide_path = U16CString::from_os_str(path.as_ref().as_os_str()).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid UTF-16 conversion: {e}"),
        )
    })?;

    let mut sid_size: u32 = SECURITY_MAX_SID_SIZE as u32;
    let mut system_sid = vec![0u8; sid_size as usize];

    unsafe {
        if CreateWellKnownSid(
            WinLocalSystemSid,
            ptr::null_mut(),
            system_sid.as_mut_ptr().cast(),
            &mut sid_size,
        ) == 0
        {
            return Err(std::io::Error::last_os_error());
        }
    }

    let sid_ptr = system_sid.as_mut_ptr().cast();
    let acl_size = mem::size_of::<ACL>() as u32
        + mem::size_of::<ACCESS_ALLOWED_ACE>() as u32
        + unsafe { GetLengthSid(sid_ptr) };
    let mut acl_buffer = vec![0u8; acl_size as usize];
    let acl = acl_buffer.as_mut_ptr() as *mut ACL;

    unsafe {
        if InitializeAcl(acl, acl_size, ACL_REVISION) == 0 {
            return Err(std::io::Error::last_os_error());
        }

        if AddAccessAllowedAceEx(
            acl,
            ACL_REVISION,
            0,
            FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            sid_ptr.cast(),
        ) == 0
        {
            return Err(std::io::Error::last_os_error());
        }
    }

    let mut security_desc: SECURITY_DESCRIPTOR = unsafe { mem::zeroed() };

    unsafe {
        if InitializeSecurityDescriptor(
            &mut security_desc as *mut _ as *mut _,
            SECURITY_DESCRIPTOR_REVISION,
        ) == 0
        {
            return Err(std::io::Error::last_os_error());
        }

        if SetSecurityDescriptorDacl(
            &mut security_desc as *mut _ as *mut c_void,
            TRUE,
            acl,
            FALSE,
        ) == 0
        {
            return Err(std::io::Error::last_os_error());
        }
    }

    unsafe {
        if SetNamedSecurityInfoW(
            wide_path.as_ptr() as *mut _,
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            acl,
            ptr::null_mut(),
        ) != ERROR_SUCCESS
        {
            return Err(std::io::Error::last_os_error());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn store_mnemonic() {
        let mnemonic = bip39::Mnemonic::generate_in(bip39::Language::English, 12).unwrap();
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let mnemonic_storage = OnDiskMnemonicStorage::new(path.clone());
        mnemonic_storage
            .store_mnemonic(mnemonic.clone())
            .await
            .unwrap();

        let stored_mnemonic = mnemonic_storage.load_mnemonic().await.unwrap();
        assert_eq!(mnemonic, stored_mnemonic);
    }

    #[tokio::test]
    async fn store_twice_fails() {
        let mnemonic = bip39::Mnemonic::generate_in(bip39::Language::English, 12).unwrap();
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let mnemonic_storage = OnDiskMnemonicStorage::new(path.clone());
        mnemonic_storage
            .store_mnemonic(mnemonic.clone())
            .await
            .unwrap();

        let result = mnemonic_storage.store_mnemonic(mnemonic).await;
        assert!(matches!(
            result,
            Err(OnDiskMnemonicStorageError::MnemonicAlreadyStored { .. })
        ));
    }

    #[tokio::test]
    async fn load_fails_if_file_does_not_exist() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let mnemonic_storage = OnDiskMnemonicStorage::new(path.clone());
        let result = mnemonic_storage.load_mnemonic().await;
        assert!(matches!(
            result,
            Err(OnDiskMnemonicStorageError::FileOpenError(_))
        ));
    }

    #[tokio::test]
    async fn load_fails_if_no_mnemonic_file() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let mnemonic_storage = OnDiskMnemonicStorage::new(path.clone());
        let result = mnemonic_storage.load_mnemonic().await;
        assert!(matches!(
            result,
            Err(OnDiskMnemonicStorageError::FileOpenError(_))
        ));
    }

    #[tokio::test]
    async fn load_fails_if_no_mnemonic_stored() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let mnemonic_storage = OnDiskMnemonicStorage::new(path.clone());
        let _ = File::create(&path).unwrap();
        let result = mnemonic_storage.load_mnemonic().await;
        assert!(matches!(
            result,
            Err(OnDiskMnemonicStorageError::ReadError(_))
        ));
    }
}
