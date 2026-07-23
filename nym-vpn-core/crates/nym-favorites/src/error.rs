// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::TunnelType;

#[derive(thiserror::Error, Debug)]
pub enum RecentsError {
    #[error("failed to lookup gateway cache")]
    GetGateways {
        tunnel_type: TunnelType,
        source: nym_gateway_directory::Error,
    },
}
