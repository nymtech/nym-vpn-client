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
}
