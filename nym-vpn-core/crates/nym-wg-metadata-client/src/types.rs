// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_wireguard_private_metadata_shared::{v1, v2};

#[derive(Debug, Clone, Copy)]
pub struct AvailableBandwidth {
    pub bandwidth_bytes: i64,
    pub upgrade_mode: Option<bool>,
}

impl From<v1::AvailableBandwidthResponse> for AvailableBandwidth {
    fn from(value: v1::AvailableBandwidthResponse) -> Self {
        AvailableBandwidth {
            bandwidth_bytes: value.available_bandwidth,
            upgrade_mode: None,
        }
    }
}

impl From<v2::AvailableBandwidthResponse> for AvailableBandwidth {
    fn from(value: v2::AvailableBandwidthResponse) -> Self {
        AvailableBandwidth {
            bandwidth_bytes: value.available_bandwidth,
            upgrade_mode: Some(value.upgrade_mode),
        }
    }
}

impl From<v1::TopUpResponse> for AvailableBandwidth {
    fn from(value: v1::TopUpResponse) -> Self {
        AvailableBandwidth {
            bandwidth_bytes: value.available_bandwidth,
            upgrade_mode: None,
        }
    }
}

impl From<v2::TopUpResponse> for AvailableBandwidth {
    fn from(value: v2::TopUpResponse) -> Self {
        AvailableBandwidth {
            bandwidth_bytes: value.available_bandwidth,
            upgrade_mode: Some(value.upgrade_mode),
        }
    }
}
