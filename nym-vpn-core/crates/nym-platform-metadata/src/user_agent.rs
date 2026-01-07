// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

// Re-exports used by new_user_agent macros
#[doc(hidden)]
pub use nym_bin_common;

/// Macro that creates `nym_sdk::UserAgent` from compiled in vergen metadata.
/// Crates using this macro should have vergen added to their `build.rs`
#[macro_export]
macro_rules! new_user_agent {
    () => {{
        let bin_info = $crate::user_agent::nym_bin_common::bin_info_local_vergen!();
        let sys_info = $crate::SysInfo::new();
        let platform = format!(
            "{}; {}; {}",
            sys_info.system_name, sys_info.os_version, sys_info.arch
        );
        ::nym_sdk::UserAgent {
            application: bin_info.binary_name.to_string(),
            version: bin_info.build_version.to_string(),
            platform,
            git_commit: bin_info.commit_sha.to_string(),
        }
    }};
}
