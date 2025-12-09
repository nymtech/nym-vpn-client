// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::proto;

impl From<proto::NetworkStatsConfig> for nym_vpn_lib_types::NetworkStatisticsConfig {
    fn from(config: proto::NetworkStatsConfig) -> Self {
        Self {
            enabled: config.enabled,
            allow_disconnected: config.allow_disconnected,
        }
    }
}
impl From<nym_vpn_lib_types::NetworkStatisticsConfig> for proto::NetworkStatsConfig {
    fn from(config: nym_vpn_lib_types::NetworkStatisticsConfig) -> Self {
        Self {
            enabled: config.enabled,
            allow_disconnected: config.allow_disconnected,
        }
    }
}

impl From<proto::NetworkStatisticsIdentity> for nym_vpn_lib_types::NetworkStatisticsIdentity {
    fn from(identity: proto::NetworkStatisticsIdentity) -> Self {
        Self {
            seed: identity.seed,
            id: identity.id,
        }
    }
}

impl From<nym_vpn_lib_types::NetworkStatisticsIdentity> for proto::NetworkStatisticsIdentity {
    fn from(identity: nym_vpn_lib_types::NetworkStatisticsIdentity) -> Self {
        Self {
            seed: identity.seed,
            id: identity.id,
        }
    }
}

impl From<proto::PrivyDerivationMessage> for nym_vpn_lib_types::PrivyDerivationMessage {
    fn from(message: proto::PrivyDerivationMessage) -> Self {
        Self {
            message: message.message,
        }
    }
}

impl From<nym_vpn_lib_types::PrivyDerivationMessage> for proto::PrivyDerivationMessage {
    fn from(message: nym_vpn_lib_types::PrivyDerivationMessage) -> Self {
        Self {
            message: message.message,
        }
    }
}
