// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::response::{NymVpnDevice, NymVpnDeviceStatus, NymVpnUsage};

use crate::{
    get_account_usage_response::{
        AccountUsage as ProtoAccountUsage, AccountUsages as ProtoAccountUsages,
    },
    get_devices_response::{
        device::DeviceStatus as ProtoDeviceStatus, Device as ProtoDevice, Devices as ProtoDevices,
    },
};

impl From<NymVpnUsage> for ProtoAccountUsage {
    fn from(usage: NymVpnUsage) -> Self {
        Self {
            created_on_utc: usage.created_on_utc,
            last_updated_utc: usage.last_updated_utc,
            id: usage.id,
            subscription_id: usage.subscription_id,
            valid_until_utc: usage.valid_until_utc,
            valid_from_utc: usage.valid_from_utc,
            bandwidth_allowance_gb: usage.bandwidth_allowance_gb,
            bandwidth_used_gb: usage.bandwidth_used_gb,
        }
    }
}

impl From<Vec<NymVpnUsage>> for ProtoAccountUsages {
    fn from(usage: Vec<NymVpnUsage>) -> Self {
        Self {
            account_usages: usage.into_iter().map(ProtoAccountUsage::from).collect(),
        }
    }
}

impl From<NymVpnDeviceStatus> for ProtoDeviceStatus {
    fn from(value: NymVpnDeviceStatus) -> Self {
        match value {
            NymVpnDeviceStatus::Active => Self::Active,
            NymVpnDeviceStatus::Inactive => Self::Inactive,
            NymVpnDeviceStatus::DeleteMe => Self::DeleteMe,
        }
    }
}

impl From<NymVpnDevice> for ProtoDevice {
    fn from(device: NymVpnDevice) -> Self {
        Self {
            created_on_utc: device.created_on_utc,
            last_updated_utc: device.last_updated_utc,
            device_identity_key: device.device_identity_key,
            status: ProtoDeviceStatus::from(device.status) as i32,
        }
    }
}

impl From<Vec<NymVpnDevice>> for ProtoDevices {
    fn from(devices: Vec<NymVpnDevice>) -> Self {
        Self {
            devices: devices.into_iter().map(ProtoDevice::from).collect(),
        }
    }
}
