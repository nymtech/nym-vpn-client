// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CommandInterfaceError {
    #[error("Failed to parse DNS IP address: {ip}")]
    ParseDnsIp {
        ip: String,
        source: std::net::AddrParseError,
    },

    #[error("Failed to create incoming stream at {}", socket_path.display())]
    CreateIncoming {
        socket_path: PathBuf,
        source: std::io::Error,
    },
}
