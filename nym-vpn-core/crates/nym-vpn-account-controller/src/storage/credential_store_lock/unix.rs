// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fs::{File, OpenOptions};
use std::path::Path;

use nix::errno::Errno;
use nix::fcntl::{Flock, FlockArg};

use crate::error::Error;
use crate::storage::credential_store_lock::ensure::ensure_data_dir;

const LOCK_FILE_NAME: &str = "credential_store_access.lock";

pub struct CredentialStoreAccessLock {
    _lock: Flock<File>,
}

impl CredentialStoreAccessLock {
    pub fn try_acquire(data_dir: &Path) -> Result<Self, Error> {
        Self::acquire_with(data_dir, FlockArg::LockExclusiveNonblock)
    }

    pub fn acquire_blocking(data_dir: &Path) -> Result<Self, Error> {
        Self::acquire_with(data_dir, FlockArg::LockExclusive)
    }

    fn acquire_with(data_dir: &Path, arg: FlockArg) -> Result<Self, Error> {
        ensure_data_dir(data_dir).map_err(|err| {
            tracing::warn!(
                path = %data_dir.display(),
                error = %err,
                "failed to ensure credential store data directory"
            );
            err
        })?;

        let path = data_dir.join(LOCK_FILE_NAME);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&path)
            .map_err(|err| {
                tracing::warn!(
                    path = %path.display(),
                    error = %err,
                    "failed to open credential store lock file"
                );
                Error::CredentialStoreLockIo(err)
            })?;

        let lock = Flock::lock(file, arg).map_err(|(_file, errno)| {
            if errno == Errno::EWOULDBLOCK || errno == Errno::EAGAIN {
                Error::CredentialStoreBusy
            } else {
                tracing::warn!(
                    path = %path.display(),
                    errno = %errno,
                    "failed to flock credential store lock file"
                );
                Error::CredentialStoreLockIo(std::io::Error::other(errno.to_string()))
            }
        })?;

        Ok(Self { _lock: lock })
    }
}
