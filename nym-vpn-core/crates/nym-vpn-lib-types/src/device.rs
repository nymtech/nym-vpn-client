// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct NymVpnDevice {
    pub created_on_utc: String,
    pub last_updated_utc: String,
    pub device_identity_key: String,
    pub status: NymVpnDeviceStatus,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum NymVpnDeviceStatus {
    Active,
    Inactive,
    DeleteMe,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
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

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_api_client::response::NymVpnDevice> for NymVpnDevice {
    fn from(value: nym_vpn_api_client::response::NymVpnDevice) -> Self {
        Self {
            created_on_utc: value.created_on_utc,
            last_updated_utc: value.last_updated_utc,
            device_identity_key: value.device_identity_key,
            status: NymVpnDeviceStatus::from(value.status),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_api_client::response::NymVpnDeviceStatus> for NymVpnDeviceStatus {
    fn from(value: nym_vpn_api_client::response::NymVpnDeviceStatus) -> Self {
        match value {
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

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_api_client::response::NymVpnUsage> for NymVpnUsage {
    fn from(value: nym_vpn_api_client::response::NymVpnUsage) -> Self {
        Self {
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
