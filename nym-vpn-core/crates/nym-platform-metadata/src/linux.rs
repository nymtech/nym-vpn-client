// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use sysinfo::System;

use super::command::command_stdout_lossy;

pub fn extra_metadata() -> HashMap<String, String> {
    let mut metadata = HashMap::new();

    if let Some(kernel_version) = System::kernel_version() {
        metadata.insert("kernel".to_owned(), kernel_version);
    }

    #[cfg(feature = "network-manager")]
    if let Some(nm_version) = nm_version() {
        metadata.insert("nm".to_owned(), nm_version);
    }

    if let Some(systemd_version) = systemd_version() {
        metadata.insert("systemd".to_owned(), systemd_version);
    }

    metadata
}

/// NetworkManager's version is returned as a numeric version string
/// > 1.26.0
#[cfg(feature = "network-manager")]
fn nm_version() -> Option<String> {
    let nm = nym_dbus::network_manager::NetworkManager::new().ok()?;
    nm.version_string().ok()
}

/// `systemctl --version` usually outputs two lines - one with the version, and another listing
/// features:
/// > systemd 246 (246)
/// > +PAM +AUDIT -SELINUX +IMA +APPARMOR +SMACK -SYSVINIT +UTMP +LIBCRYPTSETUP +GCRYPT -GNUTLS +ACL
fn systemd_version() -> Option<String> {
    let systemd_version_output = command_stdout_lossy("systemctl", &["--version"]).ok()?;
    let version = systemd_version_output.lines().next()?.to_string();
    Some(version)
}
