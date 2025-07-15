// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{io, net::Ipv6Addr, process::Command};

#[cfg(target_os = "linux")]
pub fn set_ipv6_addr(device_name: &str, ipv6_addr: Ipv6Addr) -> io::Result<()> {
    Command::new("ip")
        .args([
            "-6",
            "addr",
            "add",
            &ipv6_addr.to_string(),
            "dev",
            device_name,
        ])
        .output()?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn set_ipv6_addr(device_name: &str, ipv6_addr: Ipv6Addr) -> io::Result<()> {
    Command::new("ifconfig")
        .args([device_name, "inet6", "add", &ipv6_addr.to_string()])
        .output()?;
    Ok(())
}
