// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(uniffi::Record)]
pub struct NymVpnUsage {
    pub created_on_utc: String,
    pub last_updated_utc: String,
    pub id: String,
    pub subscription_id: String,
    pub valid_until_utc: String,
    pub valid_from_utc: String,
    pub bandwidth_allowance_gb: f64,
    pub bandwidth_used_gb: f64,
}

impl From<nym_vpn_api_client::response::NymVpnUsage> for NymVpnUsage {
    fn from(value: nym_vpn_api_client::response::NymVpnUsage) -> Self {
        NymVpnUsage {
            created_on_utc: value.created_on_utc,
            last_updated_utc: value.last_updated_utc,
            id: value.id,
            subscription_id: value.subscription_id,
            valid_until_utc: value.valid_until_utc,
            valid_from_utc: value.valid_from_utc,
            bandwidth_allowance_gb: value.bandwidth_allowance_gb,
            bandwidth_used_gb: value.bandwidth_used_gb,
        }
    }
}

#[derive(uniffi::Record)]
pub struct NymVpnDevice {
    pub created_on_utc: String,
    pub last_updated_utc: String,
    pub device_identity_key: String,
    pub status: NymVpnDeviceStatus,
}

#[derive(uniffi::Enum)]
pub enum NymVpnDeviceStatus {
    Active,
    Inactive,
    DeleteMe,
}

impl From<nym_vpn_api_client::response::NymVpnDeviceStatus> for NymVpnDeviceStatus {
    fn from(status: nym_vpn_api_client::response::NymVpnDeviceStatus) -> Self {
        match status {
            nym_vpn_api_client::response::NymVpnDeviceStatus::Active => NymVpnDeviceStatus::Active,
            nym_vpn_api_client::response::NymVpnDeviceStatus::Inactive => {
                NymVpnDeviceStatus::Inactive
            }
            nym_vpn_api_client::response::NymVpnDeviceStatus::DeleteMe => {
                NymVpnDeviceStatus::DeleteMe
            }
        }
    }
}

impl From<NymVpnDevice> for nym_vpn_api_client::response::NymVpnDevice {
    fn from(device: NymVpnDevice) -> Self {
        nym_vpn_api_client::response::NymVpnDevice {
            created_on_utc: device.created_on_utc,
            last_updated_utc: device.last_updated_utc,
            device_identity_key: device.device_identity_key,
            status: NymVpnDeviceStatus::from(device.status),
        }
    }
}
