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

    prefetch_zk_nyms_unlocked(data_dir, vpn_api_client, account, device, fair_usage_left).await
}

/// Fetch and store zk-nyms when the caller already holds
/// [`CredentialStoreAccessLock`] for `data_dir`.
///
/// Used by UniFFI account storage (`NymVpnAccountStorage::prefetch_zk_nyms`), which
/// acquires the lock once for the whole operation. Do not call this without holding
/// the lock unless no other accessor can touch `data_dir`.
pub async fn prefetch_zk_nyms_unlocked(
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
        other => PrefetchExternalError::Internal(other.to_string()),
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
}

/// Regression: UniFFI `prefetch_zk_nyms` holds the store lock before calling the
/// controller helper. A second nonblocking acquire in the same process must not
/// make prefetch a silent no-op.
#[cfg(all(test, unix))]
mod lock_regression {
    use super::*;
    use crate::storage::CredentialStoreAccessLock;
    use nym_vpn_api_client::types::Device;
    use nym_vpn_lib_types::{StorableAccount, StoredAccountMode};
    use nym_vpn_store::keys::device::DeviceKeys;
    use tempfile::tempdir;

    fn test_device_and_account() -> (Arc<nym_vpn_api_client::types::VpnAccount>, Device) {
        let device_keys = DeviceKeys::generate_new(&mut rand::thread_rng());
        let device = Device::from(device_keys.device_keypair().clone());
        let stored = StorableAccount {
            mnemonic: bip39::Mnemonic::parse(
                "dash hungry rate famous lesson march suit refuse excite soul faith bid buddy tortoise melody advice dirt coffee fluid sure air decrease cargo work",
            )
            .expect("mnemonic"),
            mode: StoredAccountMode::Api,
        };
        let account = Arc::new(
            nym_vpn_api_client::types::VpnAccount::try_from(stored).expect("vpn account"),
        );
        (account, device)
    }

    fn unreachable_api_client() -> nym_vpn_api_client::VpnApiClient {
        let urls = nym_network_defaults::ApiUrl {
            url: "http://127.0.0.1:1".to_string(),
            front_hosts: None,
        };
        nym_vpn_api_client::VpnApiClient::new(
            nym_vpn_api_client::api_urls_to_urls(&[urls]).expect("urls"),
            None,
        )
        .expect("client")
    }

    #[tokio::test]
    async fn unlocked_prefetch_does_not_reacquire_store_lock() {
        let dir = tempdir().expect("tempdir");
        let data_dir = dir.path().to_path_buf();
        let _outer_lock = CredentialStoreAccessLock::try_acquire(&data_dir).expect("outer lock");
        let (account, device) = test_device_and_account();

        let outcome = prefetch_zk_nyms_unlocked(
            data_dir,
            unreachable_api_client(),
            account,
            device,
            true,
        )
        .await;

        assert!(
            !matches!(outcome, Ok(PrefetchZkNymOutcome::SkippedStoreBusy)),
            "unlocked prefetch must not fail with SkippedStoreBusy while outer lock is held"
        );
    }

    #[tokio::test]
    async fn public_prefetch_returns_busy_when_lock_already_held() {
        let dir = tempdir().expect("tempdir");
        let data_dir = dir.path().to_path_buf();
        let _outer_lock = CredentialStoreAccessLock::try_acquire(&data_dir).expect("outer lock");
        let (account, device) = test_device_and_account();

        let outcome = prefetch_zk_nyms(
            data_dir,
            unreachable_api_client(),
            account,
            device,
            true,
        )
        .await
        .expect("prefetch result");

        assert_eq!(outcome, PrefetchZkNymOutcome::SkippedStoreBusy);
    }
}
