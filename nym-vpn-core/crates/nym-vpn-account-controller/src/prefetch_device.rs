// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Device registration gate for one-shot zk-nym prefetch (iOS login path).

use nym_vpn_api_client::{
    VpnApiClient,
    types::{Device, VpnAccount},
};
use nym_vpn_lib_types::VpnAccountSummary;

use crate::error::Error;

pub const MAX_DEVICES_REACHED: &str = "Maximum number of devices reached";
pub const FAIR_USAGE_DEPLETED: &str = "Fair usage depleted";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceRegistrationReadiness {
    AlreadyRegistered,
    MustRegister,
}

pub fn device_registration_readiness(
    summary: &VpnAccountSummary,
) -> Result<DeviceRegistrationReadiness, Error> {
    if summary.is_device_active {
        return Ok(DeviceRegistrationReadiness::AlreadyRegistered);
    }
    if summary.remaining_devices == 0 {
        return Err(Error::internal(MAX_DEVICES_REACHED));
    }
    if !summary.fair_usage_left() {
        return Err(Error::internal(FAIR_USAGE_DEPLETED));
    }
    Ok(DeviceRegistrationReadiness::MustRegister)
}

pub async fn register_device_for_prefetch_if_needed(
    vpn_api_client: &VpnApiClient,
    account: &VpnAccount,
    device: &Device,
    summary: &mut VpnAccountSummary,
) -> Result<(), Error> {
    match device_registration_readiness(summary)? {
        DeviceRegistrationReadiness::AlreadyRegistered => Ok(()),
        DeviceRegistrationReadiness::MustRegister => {
            tracing::info!(
                "Registering device {} for account {} before zk-nym prefetch",
                device.identity_key(),
                account.id()
            );
            vpn_api_client
                .register_device(account, device)
                .await
                .map_err(|err| Error::internal(format!("Failed to register device: {err}")))?;

            summary.is_device_active = true;
            summary.remaining_devices = summary.remaining_devices.saturating_sub(1);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nym_vpn_lib_types::{
        NymVpnSubscription, NymVpnSubscriptionKind, NymVpnSubscriptionStatus, Subscription,
        VpnAccountStatus,
    };
    use time::OffsetDateTime;

    fn active_subscription() -> Subscription {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        Subscription {
            status: NymVpnSubscriptionStatus::Active,
            subscription: NymVpnSubscription {
                created_on_utc: "2024-01-01T00:00:00Z".into(),
                last_updated_utc: "2024-01-01T00:00:00Z".into(),
                id: "sub".into(),
                valid_from_utc: now - 86_400,
                valid_until_utc: now + 365 * 86_400,
                status: "active".into(),
                kind: NymVpnSubscriptionKind::OneMonth,
                is_recurring: false,
            },
        }
    }

    fn summary(
        is_device_active: bool,
        remaining_devices: u64,
        traffic_used_gb: u64,
    ) -> VpnAccountSummary {
        VpnAccountSummary {
            traffic_used_gb,
            traffic_limit_gb: 2000,
            traffic_reset_time: None,
            fair_usage_data_unavailable: false,
            account_addr: "n1test".into(),
            canonical_account_addr: None,
            auth_methods: vec![],
            account_mode: None,
            subscription: Some(active_subscription()),
            is_subscription_stacked: false,
            account_status: VpnAccountStatus::Active,
            remaining_devices,
            is_device_active,
            time_synced: true,
            stale: false,
            last_synced_utc: OffsetDateTime::now_utc(),
        }
    }

    #[test]
    fn already_registered_device_skips_registration() {
        let readiness = device_registration_readiness(&summary(true, 0, 0)).unwrap();
        assert_eq!(readiness, DeviceRegistrationReadiness::AlreadyRegistered);
    }

    #[test]
    fn inactive_device_with_slots_must_register() {
        let readiness = device_registration_readiness(&summary(false, 2, 0)).unwrap();
        assert_eq!(readiness, DeviceRegistrationReadiness::MustRegister);
    }

    #[test]
    fn max_devices_blocks_registration() {
        let err = device_registration_readiness(&summary(false, 0, 0)).unwrap_err();
        assert!(err.to_string().contains(MAX_DEVICES_REACHED));
    }

    #[test]
    fn fair_usage_depleted_blocks_registration() {
        let err = device_registration_readiness(&summary(false, 2, 2000)).unwrap_err();
        assert!(err.to_string().contains(FAIR_USAGE_DEPLETED));
    }
}
