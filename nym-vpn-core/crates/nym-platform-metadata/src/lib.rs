// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use sha2::{Digest, Sha256};
use std::{env, fmt::Display};
use sysinfo::System;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(any(target_os = "macos", target_os = "ios"))]
#[path = "apple.rs"]
mod imp;

#[cfg(windows)]
#[path = "windows.rs"]
mod imp;

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use imp::AppleVersion;
#[cfg(windows)]
pub use imp::WindowsVersion;
pub use imp::{extra_metadata, short_version, version};

#[derive(Debug, Clone)]
pub struct SysInfo {
    pub os_version: String,
    pub kernel_version: String,
    pub arch: String,
    pub extra: Vec<String>,
}

impl SysInfo {
    pub fn new() -> Self {
        let os_version = System::long_os_version().unwrap_or_else(|| env::consts::OS.into());
        let kernel_version = System::kernel_version().unwrap_or_else(|| "unknown".to_string());
        let arch = std::env::consts::ARCH.to_string();
        let extra_metadata = extra_metadata()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>();

        SysInfo {
            os_version,
            kernel_version,
            arch,
            extra: extra_metadata,
        }
    }

    /// Generates a hash identifier based on the OS version, architecture, extra metadata, and host name.
    /// Returns a hexadecimal string representation of the hash.
    /// This identifier is used to identify the system in a way that is consistent across runs,
    /// without revealing sensitive information.
    pub fn hash_identifier(&self) -> String {
        let parts = [
            self.os_version.clone(),
            self.arch.clone(),
            self.extra.to_vec().join(" "),
            sysinfo::System::host_name().unwrap_or_else(|| "unknown".to_string()),
        ];

        let os_name = parts.join(" ");
        let hash = Sha256::digest(os_name.as_bytes());
        format!("{hash:x}")
    }
}

impl Display for SysInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "os_version: {}, arch: {}", self.os_version, self.arch)?;
        if !self.extra.is_empty() {
            write!(f, ", {}", self.extra.join(", "))?;
        }
        Ok(())
    }
}

impl Default for SysInfo {
    fn default() -> Self {
        Self::new()
    }
}
