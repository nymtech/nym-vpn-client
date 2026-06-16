// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use crate::error::Error;

pub fn ensure_data_dir(data_dir: &Path) -> Result<(), Error> {
    std::fs::create_dir_all(data_dir).map_err(Error::CredentialStoreLockIo)?;
    #[cfg(unix)]
    {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(data_dir, Permissions::from_mode(0o700))
            .map_err(Error::CredentialStoreLockIo)?;
    }
    Ok(())
}
