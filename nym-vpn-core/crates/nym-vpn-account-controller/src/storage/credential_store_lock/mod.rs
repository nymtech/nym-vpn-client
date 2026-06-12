// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Cross-process exclusive access to the credential store directory.
//!
//! The iOS app (prefetch) and network extension (account controller) share the
//! same on-disk credential DB. An advisory flock on a dedicated lock file
//! enforces temporal exclusion without relying on caller discipline alone.

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use unix::CredentialStoreAccessLock;
#[cfg(windows)]
pub use windows::CredentialStoreAccessLock;

#[cfg(not(any(unix, windows)))]
mod stub;

#[cfg(not(any(unix, windows)))]
pub use stub::CredentialStoreAccessLock;

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::error::Error;
    use tempfile::tempdir;

    #[test]
    fn second_nonblocking_acquire_returns_busy() {
        let dir = tempdir().expect("tempdir");
        let _first = CredentialStoreAccessLock::try_acquire(dir.path()).expect("first lock");
        let second = CredentialStoreAccessLock::try_acquire(dir.path());
        assert!(matches!(second, Err(Error::CredentialStoreBusy)));
    }
}
