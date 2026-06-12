// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! One-shot zk-nym prefetch usable without a running [`AccountController`].
//!
//! A client (e.g. the iOS app at login) can top up the local credential store
//! on the SAME on-disk path the tunnel's controller reads at connect time, so
//! the first connect skips the zk-nym fetch during `AwaitingAccountReadiness`.
//!
//! Caller invariant: opens credential storage at `data_dir`. Cross-process
//! exclusion is enforced via [`CredentialStoreAccessLock`] before storage setup.

use std::{path::PathBuf, sync::Arc};

use nym_vpn_api_client::{
    VpnApiClient,
    types::{Device, VpnAccount},
};

use crate::{
    error::Error,
    state_machine::{RequestingZkNymsState, ZkNymError, ZkNymFetchResult},
    storage::{CredentialStoreAccessLock, VpnCredentialStorage},
};

/// Result of a one-shot prefetch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefetchZkNymOutcome {
    /// Local storage already had enough tickets; no fetch was needed.
    SufficientBandwidth,
    /// New ticketbooks were fetched and stored.
    FetchedTickets,
    /// Upgrade mode is active; zk-nyms are not issued in this mode.
    UpgradeMode,
    /// Another process holds the credential store lock (e.g. network extension).
    SkippedStoreBusy,
}

/// Fetch and store zk-nyms into the credential store at `data_dir` if tickets
/// are running low, reusing the exact logic the account controller runs in
/// [`RequestingZkNymsState`].
///
/// `fair_usage_left` should come from the latest account summary
/// (`VpnAccountSummary::fair_usage_left`); pass `true` to attempt regardless
/// (the API rejects with a depleted code if not).
/// Prefetch zk-nyms when the caller already holds [`CredentialStoreAccessLock`]
/// on `data_dir` (e.g. UniFFI `NymVpnAccountStorage::prefetch_zk_nyms`). The flock
/// is not re-entrant; do not call [`prefetch_zk_nyms`] while holding the same lock.
pub async fn prefetch_zk_nyms_assuming_store_lock(
    data_dir: PathBuf,
    vpn_api_client: VpnApiClient,
    account: Arc<VpnAccount>,
    device: Device,
    fair_usage_left: bool,
) -> Result<PrefetchZkNymOutcome, Error> {
    let storage = VpnCredentialStorage::setup_from_path(&data_dir).await?;
    // Retain a handle to close the pool after the fetch (controller.rs:106, controller.rs:249).
    let storage_for_close = storage.clone();

    let result = RequestingZkNymsState::fetch_zk_nyms(
        vpn_api_client,
        account,
        device,
        storage,
        fair_usage_left,
    )
    .await;

    storage_for_close.close().await;

    map_fetch_result(result)
}

pub async fn prefetch_zk_nyms(
    data_dir: PathBuf,
    vpn_api_client: VpnApiClient,
    account: Arc<VpnAccount>,
    device: Device,
    fair_usage_left: bool,
) -> Result<PrefetchZkNymOutcome, Error> {
    let _store_lock = match CredentialStoreAccessLock::try_acquire(&data_dir) {
        Ok(lock) => lock,
        Err(Error::CredentialStoreBusy) => return Ok(PrefetchZkNymOutcome::SkippedStoreBusy),
        Err(err) => return Err(err),
    };

    prefetch_zk_nyms_assuming_store_lock(
        data_dir,
        vpn_api_client,
        account,
        device,
        fair_usage_left,
    )
    .await
}

fn map_fetch_result(
    result: Result<ZkNymFetchResult, ZkNymError>,
) -> Result<PrefetchZkNymOutcome, Error> {
    match result {
        Ok(ZkNymFetchResult::SufficientBandwidth) => Ok(PrefetchZkNymOutcome::SufficientBandwidth),
        Ok(ZkNymFetchResult::FetchedTickets { .. }) => Ok(PrefetchZkNymOutcome::FetchedTickets),
        Ok(ZkNymFetchResult::UpgradeMode) => Ok(PrefetchZkNymOutcome::UpgradeMode),
        Err(err) => Err(Error::PrefetchZkNym(format!("{err:?}"))),
    }
}

/// Structured error mapping for UniFFI callers (preserves lock vs zk-nym vs internal).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefetchExternalError {
    StoreBusy,
    ZkNymAcquisitionFailure(String),
    Internal(String),
}

pub fn map_prefetch_error_for_external(err: Error) -> PrefetchExternalError {
    match err {
        Error::CredentialStoreBusy => PrefetchExternalError::StoreBusy,
        Error::PrefetchZkNym(details) => PrefetchExternalError::ZkNymAcquisitionFailure(details),
        Error::Internal(details) => PrefetchExternalError::Internal(details),
        Error::SetupVpnApiClient(_)
        | Error::AccountStore { .. }
        | Error::KeyStore { .. }
        | Error::AccountSummaryStore { .. }
        | Error::StoragePaths(_)
        | Error::CredentialStorage(_)
        | Error::WireguardKeysStorage(_)
        | Error::PendingCredentialRequestsStorage(_)
        | Error::SetupCredentialStorage(_)
        | Error::SetupPendingCredentialRequestsStorage(_)
        | Error::RemoveCredentialStorage(_)
        | Error::ParseTicketType(_)
        | Error::CredentialStoreLockIo(_) => PrefetchExternalError::Internal(err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sufficient_bandwidth_maps_correctly() {
        let outcome = map_fetch_result(Ok(ZkNymFetchResult::SufficientBandwidth)).unwrap();
        assert_eq!(outcome, PrefetchZkNymOutcome::SufficientBandwidth);
    }

    #[test]
    fn fetched_tickets_maps_correctly() {
        let outcome = map_fetch_result(Ok(ZkNymFetchResult::FetchedTickets {
            types: vec!["data".to_string()],
        }))
        .unwrap();
        assert_eq!(outcome, PrefetchZkNymOutcome::FetchedTickets);
    }

    #[test]
    fn upgrade_mode_maps_correctly() {
        let outcome = map_fetch_result(Ok(ZkNymFetchResult::UpgradeMode)).unwrap();
        assert_eq!(outcome, PrefetchZkNymOutcome::UpgradeMode);
    }

    #[test]
    fn bandwidth_exceeded_maps_to_prefetch_error() {
        let result = map_fetch_result(Err(ZkNymError::BandwidthExceeded));
        assert!(result.is_err());
        let Error::PrefetchZkNym(msg) = result.unwrap_err() else {
            panic!("expected PrefetchZkNym error variant");
        };
        assert!(msg.contains("BandwidthExceeded"));
    }

    #[test]
    fn api_failure_maps_to_prefetch_error() {
        let result = map_fetch_result(Err(ZkNymError::ApiFailure("503".to_string())));
        assert!(result.is_err());
        let Error::PrefetchZkNym(msg) = result.unwrap_err() else {
            panic!("expected PrefetchZkNym error variant");
        };
        assert!(msg.contains("ApiFailure"));
    }

    #[test]
    fn skipped_store_busy_is_distinct_outcome() {
        assert_eq!(
            PrefetchZkNymOutcome::SkippedStoreBusy,
            PrefetchZkNymOutcome::SkippedStoreBusy
        );
    }

    #[test]
    fn maps_credential_store_busy_for_external_callers() {
        assert_eq!(
            map_prefetch_error_for_external(Error::CredentialStoreBusy),
            PrefetchExternalError::StoreBusy
        );
    }

    #[test]
    fn maps_prefetch_zknym_for_external_callers() {
        assert_eq!(
            map_prefetch_error_for_external(Error::PrefetchZkNym("BandwidthExceeded".into())),
            PrefetchExternalError::ZkNymAcquisitionFailure("BandwidthExceeded".into())
        );
    }

    #[test]
    fn maps_internal_for_external_callers() {
        assert_eq!(
            map_prefetch_error_for_external(Error::internal("Device time is desynced")),
            PrefetchExternalError::Internal("Device time is desynced".into())
        );
    }

    #[test]
    fn maps_unexpected_prefetch_errors_to_internal() {
        assert_eq!(
            map_prefetch_error_for_external(Error::ParseTicketType("mixnet".into())),
            PrefetchExternalError::Internal("failed to parse ticket type: mixnet".into())
        );
    }

    #[tokio::test]
    async fn assuming_store_lock_does_not_return_skipped_store_busy() {
        use crate::storage::CredentialStoreAccessLock;
        use nym_vpn_api_client::api_urls_to_urls;
        use nym_vpn_store::keys::device::DeviceKeys;
        use tempfile::tempdir;

        let dir = tempdir().expect("tempdir");
        let _outer_lock = CredentialStoreAccessLock::try_acquire(dir.path()).expect("outer lock");

        let api_url = nym_network_defaults::ApiUrl {
            url: "http://127.0.0.1:1".parse().expect("url"),
            front_hosts: None,
        };
        let urls = api_urls_to_urls(&[api_url]).expect("urls");
        let user_agent = nym_http_api_client::UserAgent {
            application: "test".into(),
            version: "1.0".into(),
            platform: "test".into(),
            git_commit: "test".into(),
        };
        let vpn_api_client =
            nym_vpn_api_client::VpnApiClient::new(urls, Some(user_agent)).expect("client");

        let (account, _mnemonic) = VpnAccount::generate_new().expect("account");
        let account = Arc::new(account);
        let device_keys = DeviceKeys::generate_new(&mut rand::thread_rng());
        let device = Device::from(device_keys.device_keypair().clone());

        let result = prefetch_zk_nyms_assuming_store_lock(
            dir.path().to_path_buf(),
            vpn_api_client,
            account,
            device,
            true,
        )
        .await;

        assert!(
            !matches!(result, Ok(PrefetchZkNymOutcome::SkippedStoreBusy)),
            "assuming-store-lock path must not double-acquire and return SkippedStoreBusy"
        );
    }
}
