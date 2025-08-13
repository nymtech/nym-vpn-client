// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use sha2::{Digest, Sha256};
use std::env;
use sysinfo::System;
use tracing::info;

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

#[cfg(target_os = "android")]
pub use imp::{AndroidVersion, extra_metadata};
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use imp::{AppleVersion, extra_metadata};
#[cfg(windows)]
pub use imp::{WindowsVersion, extra_metadata};
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

    pub fn display(&self, print_extra: bool) {
        info!("os version: {}", self.os_version);
        info!("os arch: {}", self.arch);
        if print_extra {
            for info in &self.extra {
                info!("os {info}");
            }
        }
    }

    pub fn raw_display(&self, print_extra: bool) {
        println!("os version: {}", self.os_version);
        println!("os arch: {}", self.arch);
        if print_extra {
            for info in &self.extra {
                println!("os {info}");
            }
        }
    }

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

impl Default for SysInfo {
    fn default() -> Self {
        Self::new()
    }
}
