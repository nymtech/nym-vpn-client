// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub(crate) fn construct_user_agent() -> nym_vpn_lib::UserAgent {
    let bin_info = nym_bin_common::bin_info_local_vergen!();
    let name = sysinfo::System::name().unwrap_or("unknown".to_string());
    let os_long = sysinfo::System::long_os_version().unwrap_or("unknown".to_string());
    let arch = sysinfo::System::cpu_arch();
    let platform = format!("{}; {}; {}", name, os_long, arch);
    nym_vpn_lib::UserAgent {
        application: bin_info.binary_name.to_string(),
        version: bin_info.build_version.to_string(),
        platform,
        git_commit: bin_info.commit_sha.to_string(),
    }
}
