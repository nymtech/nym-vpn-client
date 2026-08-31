// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::TunnelType;

#[derive(thiserror::Error, Debug)]
pub enum FavoritesError {
    #[error("failed to lookup gateway cache")]
    GetGateways {
        tunnel_type: TunnelType,
        source: Box<nym_gateway_directory::Error>,
    },

    #[error("{0}")]
    Serde(#[from] serde_json::Error),

    #[error("{0}")]
    Io(#[from] std::io::Error),
}
