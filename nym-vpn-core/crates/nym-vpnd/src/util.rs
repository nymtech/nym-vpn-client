// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, time::Duration};

pub(crate) fn construct_user_agent() -> nym_vpn_lib::UserAgent {
    let bin_info = nym_bin_common::bin_info_local_vergen!();
    let name = sysinfo::System::name().unwrap_or("unknown".to_string());
    let os_long = sysinfo::System::long_os_version().unwrap_or("unknown".to_string());
    let arch = sysinfo::System::cpu_arch().unwrap_or("unknown".to_string());
    let platform = format!("{}; {}; {}", name, os_long, arch);
    nym_vpn_lib::UserAgent {
        application: bin_info.binary_name.to_string(),
        version: bin_info.build_version.to_string(),
        platform,
        git_commit: bin_info.commit_sha.to_string(),
    }
}

pub(crate) fn get_age_of_file(file_path: &PathBuf) -> anyhow::Result<Option<Duration>> {
    if !file_path.exists() {
        return Ok(None);
    }
    let metadata = std::fs::metadata(file_path)?;
    Ok(Some(metadata.modified()?.elapsed()?))
}
