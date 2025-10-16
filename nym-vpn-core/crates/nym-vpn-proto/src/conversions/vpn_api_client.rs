// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{NymVpnDevice, NymVpnDeviceStatus, NymVpnUsage};

use crate::{conversions::ConversionError, proto};

impl From<NymVpnUsage> for proto::get_account_usage_response::AccountUsage {
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

impl From<proto::get_account_usage_response::AccountUsage> for NymVpnUsage {
    fn from(usage: proto::get_account_usage_response::AccountUsage) -> Self {
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

impl From<Vec<NymVpnUsage>> for proto::get_account_usage_response::AccountUsages {
    fn from(usage: Vec<NymVpnUsage>) -> Self {
        Self {
            account_usages: usage
                .into_iter()
                .map(proto::get_account_usage_response::AccountUsage::from)
                .collect(),
        }
    }
}

impl From<proto::get_account_usage_response::AccountUsages> for Vec<NymVpnUsage> {
    fn from(usage: proto::get_account_usage_response::AccountUsages) -> Self {
        usage
            .account_usages
            .into_iter()
            .map(NymVpnUsage::from)
            .collect()
    }
}

impl From<NymVpnDeviceStatus> for proto::get_devices_response::device::DeviceStatus {
    fn from(value: NymVpnDeviceStatus) -> Self {
        match value {
            NymVpnDeviceStatus::Active => Self::Active,
            NymVpnDeviceStatus::Inactive => Self::Inactive,
            NymVpnDeviceStatus::DeleteMe => Self::DeleteMe,
        }
    }
}

impl From<proto::get_devices_response::device::DeviceStatus> for NymVpnDeviceStatus {
    fn from(value: proto::get_devices_response::device::DeviceStatus) -> Self {
        match value {
            proto::get_devices_response::device::DeviceStatus::Active => Self::Active,
            proto::get_devices_response::device::DeviceStatus::Inactive => Self::Inactive,
            proto::get_devices_response::device::DeviceStatus::DeleteMe => Self::DeleteMe,
        }
    }
}

impl From<NymVpnDevice> for proto::get_devices_response::Device {
    fn from(device: NymVpnDevice) -> Self {
        Self {
            created_on_utc: device.created_on_utc,
            last_updated_utc: device.last_updated_utc,
            device_identity_key: device.device_identity_key,
            status: proto::get_devices_response::device::DeviceStatus::from(device.status) as i32,
        }
    }
}

impl TryFrom<proto::get_devices_response::Device> for NymVpnDevice {
    type Error = ConversionError;

    fn try_from(device: proto::get_devices_response::Device) -> Result<Self, Self::Error> {
        let proto_device_state =
            proto::get_devices_response::device::DeviceStatus::try_from(device.status).map_err(
                |err| ConversionError::Generic(format!("failed to convert DeviceStatus: {err}")),
            )?;

        Ok(Self {
            created_on_utc: device.created_on_utc,
            last_updated_utc: device.last_updated_utc,
            device_identity_key: device.device_identity_key,
            status: NymVpnDeviceStatus::from(proto_device_state),
        })
    }
}

impl From<Vec<NymVpnDevice>> for proto::get_devices_response::Devices {
    fn from(devices: Vec<NymVpnDevice>) -> Self {
        Self {
            devices: devices
                .into_iter()
                .map(proto::get_devices_response::Device::from)
                .collect(),
        }
    }
}
