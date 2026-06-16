// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Shared account readiness checks and device registration for the NE controller
//! and one-shot iOS storage helpers (no in-app account controller).

use nym_vpn_api_client::{
    VpnApiClient,
    types::{Device, VpnAccount},
};
use nym_vpn_lib_types::VpnAccountSummary;

use crate::error::Error;

pub const MAX_DEVICES_REACHED: &str = "Maximum number of devices reached";
pub const FAIR_USAGE_DEPLETED: &str = "Fair usage depleted";
pub const DEVICE_TIME_DESYNCED: &str = "Device time is desynced";

/// How old a cached summary may be before a network refresh is required (matches
/// [`SyncingLocalState`](crate::state_machine::syncing_state::local_state::SyncingLocalState)).
pub const SUMMARY_STALE_AFTER: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceRegistrationReadiness {
    AlreadyRegistered,
    MustRegister,
}

/// Outcome of the local-sync readiness classifier used by `SyncingLocalState`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalSyncCheck {
    PendingSubscription,
    InactiveAccount(String),
    InactiveSubscription,
    MaxDevicesReached,
    FairUsageDepleted,
    DeviceTimeDesynced,
    MustRegisterDevice,
    Ready,
}

pub fn classify_local_sync(summary: &VpnAccountSummary) -> LocalSyncCheck {
    if !summary.is_account_active() {
        return LocalSyncCheck::InactiveAccount(summary.account_status.to_string());
    }
    if summary.is_subscription_pending() {
        return LocalSyncCheck::PendingSubscription;
    }
    if !summary.is_subscription_active() {
        return LocalSyncCheck::InactiveSubscription;
    }
    if !summary.is_device_active {
        if summary.remaining_devices == 0 {
            return LocalSyncCheck::MaxDevicesReached;
        }
        if !summary.fair_usage_left() {
            return LocalSyncCheck::FairUsageDepleted;
        }
        return LocalSyncCheck::MustRegisterDevice;
    }
    if !summary.time_synced {
        return LocalSyncCheck::DeviceTimeDesynced;
    }
    LocalSyncCheck::Ready
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

pub fn verify_time_synced(summary: &VpnAccountSummary) -> Result<(), Error> {
    if summary.time_synced {
        Ok(())
    } else {
        Err(Error::internal(DEVICE_TIME_DESYNCED))
    }
}

/// When a device row is active, time sync must pass before post-login setup succeeds.
pub fn validate_active_device_time_sync(summary: &VpnAccountSummary) -> Result<(), Error> {
    if summary.is_device_active {
        verify_time_synced(summary)?;
    }
    Ok(())
}

/// Network-first account summary read with cache fallback on transient failure.
pub fn account_summary_after_network_error<E: Clone>(
    network_err: E,
    cached: Option<VpnAccountSummary>,
    is_fatal: impl FnOnce(&E) -> bool,
) -> Result<Option<VpnAccountSummary>, E> {
    if is_fatal(&network_err) {
        return Err(network_err);
    }
    match cached {
        Some(summary) => Ok(Some(summary)),
        None => Err(network_err),
    }
}

pub async fn register_device_for_account(
    vpn_api_client: &VpnApiClient,
    account: &VpnAccount,
    device: &Device,
    summary: &mut VpnAccountSummary,
) -> Result<(), Error> {
    tracing::info!(
        "Registering device {} for account {}",
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

/// Register the device when the summary shows a free slot. Returns `true` when a
/// new registration POST was performed.
pub async fn register_device_if_needed(
    vpn_api_client: &VpnApiClient,
    account: &VpnAccount,
    device: &Device,
    summary: &mut VpnAccountSummary,
) -> Result<bool, Error> {
    match device_registration_readiness(summary)? {
        DeviceRegistrationReadiness::AlreadyRegistered => Ok(false),
        DeviceRegistrationReadiness::MustRegister => {
            register_device_for_account(vpn_api_client, account, device, summary).await?;
            Ok(true)
        }
    }
}

/// Back-compat alias for prefetch call sites.
pub async fn register_device_for_prefetch_if_needed(
    vpn_api_client: &VpnApiClient,
    account: &VpnAccount,
    device: &Device,
    summary: &mut VpnAccountSummary,
) -> Result<(), Error> {
    register_device_if_needed(vpn_api_client, account, device, summary)
        .await
        .map(|_| ())
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
        time_synced: bool,
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
            time_synced,
            stale: false,
            last_synced_utc: OffsetDateTime::now_utc(),
        }
    }

    #[test]
    fn already_registered_device_skips_registration() {
        let readiness = device_registration_readiness(&summary(true, 0, 0, true)).unwrap();
        assert_eq!(readiness, DeviceRegistrationReadiness::AlreadyRegistered);
    }

    #[test]
    fn inactive_device_with_slots_must_register() {
        let readiness = device_registration_readiness(&summary(false, 2, 0, true)).unwrap();
        assert_eq!(readiness, DeviceRegistrationReadiness::MustRegister);
    }

    #[test]
    fn max_devices_blocks_registration() {
        let err = device_registration_readiness(&summary(false, 0, 0, true)).unwrap_err();
        assert!(err.to_string().contains(MAX_DEVICES_REACHED));
    }

    #[test]
    fn fair_usage_depleted_blocks_registration() {
        let err = device_registration_readiness(&summary(false, 2, 2000, true)).unwrap_err();
        assert!(err.to_string().contains(FAIR_USAGE_DEPLETED));
    }

    #[test]
    fn classify_local_sync_ready_when_device_active_and_time_synced() {
        assert_eq!(
            classify_local_sync(&summary(true, 1, 0, true)),
            LocalSyncCheck::Ready
        );
    }

    #[test]
    fn classify_local_sync_must_register_device() {
        assert_eq!(
            classify_local_sync(&summary(false, 2, 0, true)),
            LocalSyncCheck::MustRegisterDevice
        );
    }

    #[test]
    fn classify_local_sync_device_time_desynced() {
        assert_eq!(
            classify_local_sync(&summary(true, 1, 0, false)),
            LocalSyncCheck::DeviceTimeDesynced
        );
    }

    #[test]
    fn verify_time_synced_rejects_desynced_summary() {
        let err = verify_time_synced(&summary(true, 1, 0, false)).unwrap_err();
        assert!(err.to_string().contains(DEVICE_TIME_DESYNCED));
    }

    #[test]
    fn validate_active_device_time_sync_rejects_desynced_active_device() {
        let err = validate_active_device_time_sync(&summary(true, 1, 0, false)).unwrap_err();
        assert!(err.to_string().contains(DEVICE_TIME_DESYNCED));
    }

    #[test]
    fn validate_active_device_time_sync_skips_inactive_device() {
        assert!(validate_active_device_time_sync(&summary(false, 2, 0, false)).is_ok());
    }

    #[test]
    fn post_login_setup_classifies_desynced_active_device() {
        assert_eq!(
            classify_local_sync(&summary(true, 1, 0, false)),
            LocalSyncCheck::DeviceTimeDesynced
        );
        let err = validate_active_device_time_sync(&summary(true, 1, 0, false)).unwrap_err();
        assert!(err.to_string().contains(DEVICE_TIME_DESYNCED));
    }

    #[test]
    fn prefetch_registration_decision_uses_fresh_inactive_summary() {
        let stale_cache_would_skip = summary(true, 1, 0, true);
        let fresh_network_requires_register = summary(false, 2, 0, true);
        assert_eq!(
            device_registration_readiness(&stale_cache_would_skip).unwrap(),
            DeviceRegistrationReadiness::AlreadyRegistered
        );
        assert_eq!(
            device_registration_readiness(&fresh_network_requires_register).unwrap(),
            DeviceRegistrationReadiness::MustRegister
        );
    }

    #[test]
    fn post_login_setup_blocks_on_fair_usage_depleted() {
        assert_eq!(
            classify_local_sync(&summary(false, 2, 2000, true)),
            LocalSyncCheck::FairUsageDepleted
        );
    }

    #[test]
    fn account_summary_network_error_returns_cache_when_present() {
        let cached = summary(true, 1, 0, true);
        let result =
            account_summary_after_network_error("network down", Some(cached.clone()), |_| false)
                .expect("cached summary");
        assert_eq!(result, Some(cached));
    }

    #[test]
    fn account_summary_network_error_without_cache_propagates() {
        let err = "network down";
        let result = account_summary_after_network_error(err, None, |_| false);
        assert_eq!(result, Err(err));
    }

    #[test]
    fn account_summary_fatal_errors_skip_cache_fallback() {
        let cached = summary(true, 1, 0, true);
        let result =
            account_summary_after_network_error("no account", Some(cached), |e| *e == "no account");
        assert_eq!(result, Err("no account"));
    }
}
