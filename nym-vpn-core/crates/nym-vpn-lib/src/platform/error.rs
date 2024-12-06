// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_tunnel_provider::error::VpnError;

impl From<crate::Error> for VpnError {
    fn from(value: crate::Error) -> Self {
        Self::InternalError {
            details: value.to_string(),
        }
    }
}
