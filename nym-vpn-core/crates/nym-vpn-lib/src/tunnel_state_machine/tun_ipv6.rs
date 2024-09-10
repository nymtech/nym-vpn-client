// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{io, net::Ipv6Addr};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::process::Command;

pub fn set_ipv6_addr(_device_name: &str, _ipv6_addr: Ipv6Addr) -> io::Result<()> {
    #[cfg(target_os = "linux")]
    Command::new("ip")
        .args([
            "-6",
            "addr",
            "add",
            &_ipv6_addr.to_string(),
            "dev",
            _device_name,
        ])
        .output()?;

    #[cfg(target_os = "macos")]
    Command::new("ifconfig")
        .args([_device_name, "inet6", "add", &_ipv6_addr.to_string()])
        .output()?;

    Ok(())
}
