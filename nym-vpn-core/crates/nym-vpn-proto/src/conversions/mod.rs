// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod account;
pub mod account_controller_state;
pub mod diagnostic;
pub mod error;
pub mod gateway_directory;
pub mod gateway_independence;
pub mod gateway_selection_algorithm;
pub mod network_config;
pub mod network_stats;
pub mod prost;
pub mod recents;
pub mod service;
pub mod socket_addr;
pub mod socks5;
pub mod split_tunnel;
pub mod split_tunnel_settings;
pub mod tunnel_event;
pub mod tunnel_state;
pub mod vpn_api_client;
pub mod vpnd;

pub use error::ConversionError;

use crate::proto;

impl From<proto::UserAgent> for nym_vpn_lib_types::UserAgent {
    fn from(user_agent: proto::UserAgent) -> Self {
        Self {
            application: user_agent.application,
            version: user_agent.version,
            platform: user_agent.platform,
            git_commit: user_agent.git_commit,
        }
    }
}

impl From<nym_vpn_lib_types::UserAgent> for proto::UserAgent {
    fn from(user_agent: nym_vpn_lib_types::UserAgent) -> Self {
        Self {
            application: user_agent.application,
            version: user_agent.version,
            platform: user_agent.platform,
            git_commit: user_agent.git_commit,
        }
    }
}

impl From<std::net::IpAddr> for proto::Dns {
    fn from(ip: std::net::IpAddr) -> Self {
        Self {
            ip: Some(ip.to_string()),
        }
    }
}

impl From<u8> for proto::Threshold {
    fn from(performance: u8) -> Self {
        Self {
            min_performance: performance.into(),
        }
    }
}
