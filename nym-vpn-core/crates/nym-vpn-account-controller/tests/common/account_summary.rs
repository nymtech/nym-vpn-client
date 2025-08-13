// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::response::{
    NymErrorResponse, NymVpnAccountResponse, NymVpnAccountStatusResponse,
    NymVpnAccountSummaryDevices, NymVpnAccountSummaryFairUsage, NymVpnAccountSummaryResponse,
    NymVpnAccountSummarySubscription, NymVpnAccountSummaryWithDeviceResponse, NymVpnDevice,
    NymVpnDeviceStatus, NymVpnSubscription, NymVpnSubscriptionKind, NymVpnSubscriptionStatus,
};

const MAX_DEVICE_NB: u64 = 10;
const MAX_FAIR_USAGE: u64 = 2000;

pub fn mock_subscription_summary_with_device(
    active_account: bool,
    active_sub: bool,
    device_nb: u64,
    fair_usage: u64,
    active_device: Option<NymVpnDevice>,
) -> NymVpnAccountSummaryWithDeviceResponse {
    let subscription = if active_sub {
        Some(NymVpnSubscription {
            created_on_utc: "whatever".to_string(),
            last_updated_utc: "whatever".to_string(),
            id: "whatever".to_string(),
            valid_until_utc: "2025-08-20 13:46:26.572Z".to_string(),
            valid_from_utc: "2025-07-21 13:46:26.572Z".to_string(),
            status: NymVpnSubscriptionStatus::Active,
            kind: NymVpnSubscriptionKind::Freepass,
        })
    } else {
        None
    };

    let account_status = if active_account {
        NymVpnAccountStatusResponse::Active
    } else {
        NymVpnAccountStatusResponse::Inactive
    };

    NymVpnAccountSummaryWithDeviceResponse {
        account_summary: NymVpnAccountSummaryResponse {
            account: NymVpnAccountResponse {
                created_on_utc: "whatever".to_string(),
                last_updated_utc: "whatever".to_string(),
                account_addr: "whatever".to_string(),
                status: account_status,
            },
            subscription: NymVpnAccountSummarySubscription {
                is_active: active_sub,
                active: subscription,
            },
            devices: NymVpnAccountSummaryDevices {
                active: device_nb,
                max: MAX_DEVICE_NB,
                remaining: MAX_DEVICE_NB - device_nb,
            },
            fair_usage: NymVpnAccountSummaryFairUsage {
                usedGB: fair_usage,
                limitGB: MAX_FAIR_USAGE,
                resetsOnUtc: Some("2025-08-20 13:46:26.572Z".to_string()),
            },
        },
        active_device,
    }
}

pub fn mock_api_device(status: NymVpnDeviceStatus) -> NymVpnDevice {
    NymVpnDevice {
        created_on_utc: "whatever".to_string(),
        last_updated_utc: "whatever".to_string(),
        device_identity_key: "whatever".to_string(),
        status,
    }
}

pub fn inactive_account() -> NymVpnAccountSummaryWithDeviceResponse {
    mock_subscription_summary_with_device(false, false, 0, 0, None)
}

pub fn account_with_inactive_sub() -> NymVpnAccountSummaryWithDeviceResponse {
    mock_subscription_summary_with_device(true, false, 0, 0, None)
}

pub fn account_with_unregistered_device() -> NymVpnAccountSummaryWithDeviceResponse {
    mock_subscription_summary_with_device(true, true, 0, 0, None)
}
pub fn account_ready_to_connect() -> NymVpnAccountSummaryWithDeviceResponse {
    mock_subscription_summary_with_device(
        true,
        true,
        0,
        0,
        Some(mock_api_device(NymVpnDeviceStatus::Active)),
    )
}

// Special cases
pub fn account_no_fair_usage() -> NymVpnAccountSummaryWithDeviceResponse {
    mock_subscription_summary_with_device(true, true, 0, MAX_FAIR_USAGE, None)
}

pub fn account_max_devices() -> NymVpnAccountSummaryWithDeviceResponse {
    mock_subscription_summary_with_device(true, true, MAX_DEVICE_NB, 0, None)
}

pub fn unregistered_account() -> NymErrorResponse {
    NymErrorResponse {
        message: "Account not found".to_string(),
        message_id: None,
        code_reference_id: None,
        status: "access_denied".to_string(),
    }
}

pub fn unrelated_error() -> NymErrorResponse {
    NymErrorResponse {
        message: "Access denied".to_string(),
        message_id: Some("nym-vpn-website.public-api.access-denied".to_string()),
        code_reference_id: Some("55cbd0ee-4ff5-4f3d-930e-6f6a95ce849f".to_string()),
        status: "access_denied".to_string(),
    }
}
