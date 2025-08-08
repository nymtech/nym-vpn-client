// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{io, net::IpAddr};
use tokio::process::Command;

/// Adds an alias to a network interface.
pub async fn add_alias(interface: &str, addr: IpAddr) -> io::Result<()> {
    let output = Command::new("ifconfig")
        .args([interface, "alias", &addr.to_string(), "up"])
        .output()
        .await?;

    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "Non-zero exit code from ifconfig: {}",
            output.status
        )))
    }
}

/// Removes an alias from a network interface.
pub async fn remove_alias(interface: &str, addr: IpAddr) -> io::Result<()> {
    let output = Command::new("ifconfig")
        .args([interface, "delete", &format!("{addr}")])
        .output()
        .await?;

    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "Non-zero exit code from ifconfig: {}",
            output.status
        )))
    }
}
