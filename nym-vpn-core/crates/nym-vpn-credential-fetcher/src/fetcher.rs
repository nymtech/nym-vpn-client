// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use nym_bandwidth_controller::{
    CredentialFetcher, CredentialFetcherError, CredentialPublicDataFetcher, NymCredential,
};
use nym_credentials::{
    AggregatedCoinIndicesSignatures, AggregatedExpirationDateSignatures, EpochVerificationKey,
};
use nym_credentials_interface::TicketType;
use nym_validator_client::nym_api::EpochId;
use nym_vpn_api_client::{
    VpnApiClient,
    types::{Device, VpnAccount},
};
use time::Date;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

use crate::{
    cached_data::CachedData, credential_request::CredentialRequestTask, error::VpnApiFetcherError,
    storage::PendingCredentialRequestsStorage, utils::with_retries,
};

// Little enum to avoid a boolean in the pause/resume logic
/// Pause gate: `While paused, all fetch operations stall (and any in-flight
/// one is cancelled and restarted on resume)
#[derive(PartialEq, Eq)]
enum Status {
    Paused,
    Running,
}

/// A [`CredentialFetcher`] that acquires zk-nym ticketbooks from the Nym VPN API.
///
/// Bind one to a specific account + device; install it on the
/// [`BandwidthController`](nym_bandwidth_controller::BandwidthController) via
/// `set_credential_fetcher`, and replace/unset it when the account or device changes.
pub struct VpnApiCredentialFetcher {
    vpn_api_client: VpnApiClient,
    account: Arc<VpnAccount>,
    device: Device,
    pending_storage: PendingCredentialRequestsStorage,
    cached_data: CachedData,

    status_tx: watch::Sender<Status>,
    /// Interrupts any in-flight (or stalled) fetch and makes it return.
    cancellation_token: CancellationToken,
}

impl VpnApiCredentialFetcher {
    /// Build a fetcher.
    ///
    /// - `data_dir`: directory in which to create this fetcher's own storage for resuming in-flight
    ///   requests.
    /// - `cancellation_token`: cancelling it interrupts any in-flight or stalled fetch
    pub async fn new(
        vpn_api_client: VpnApiClient,
        account: Arc<VpnAccount>,
        device: Device,
        data_dir: impl AsRef<Path>,
        cancellation_token: CancellationToken,
    ) -> Result<Self, VpnApiFetcherError> {
        let pending_storage = PendingCredentialRequestsStorage::init(data_dir).await?;
        let cached_data = CachedData::new(vpn_api_client.clone());
        let (status_tx, _) = watch::channel(Status::Running);
        Ok(VpnApiCredentialFetcher {
            vpn_api_client,
            account,
            device,
            pending_storage,
            cached_data,
            status_tx,
            cancellation_token,
        })
    }

    /// Pause the fetcher. Any in-flight [`CredentialFetcher`]/[`CredentialPublicDataFetcher`] call is
    /// cancelled, and new ones stall, until [`resume`](Self::resume) is called. Calls never error
    /// because of a pause; they simply hang and then restart on resume.
    pub fn pause(&self) {
        let _ = self.status_tx.send(Status::Paused);
    }

    /// Resume the fetcher, releasing any stalled calls and restarting any that were cancelled by a
    /// [`pause`](Self::pause).
    pub fn resume(&self) {
        let _ = self.status_tx.send(Status::Running);
    }

    /// Run `op` but gate it on the pause state: stall until not paused, then run it; if a
    /// pause happens mid-flight, drop the in-flight work and restart it once resumed. A pause never
    /// returns (no error, no premature value). Cancellation, in contrast, interrupts the in-flight
    /// (or stalled) work and returns `VpnApiFetcherError::Cancelled` — the fetcher is being torn down.
    async fn run_while_active<Fut, T>(
        &self,
        mut op: impl FnMut() -> Fut,
    ) -> Result<T, VpnApiFetcherError>
    where
        Fut: std::future::Future<Output = T>,
    {
        self.cancellation_token
            .run_until_cancelled(async {
                loop {
                    self.wait_for_status(Status::Running).await;

                    tokio::select! {
                        biased; // If the result is there just when we're paused/cancelled, we want to keep it
                        result = op() => return Ok(result),
                        // Interruption. Leaving the select drops the future and waits for a running status again
                        _ = self.wait_for_status(Status::Paused) => {
                            tracing::debug!("VPN-API fetcher interrupted");
                        },
                    }
                }
            })
            .await
            .ok_or(VpnApiFetcherError::Cancelled)?
    }

    // While we could technically avoid the self borrow, it guarantees that the sender lives so we can unwrap the `wait_for` call
    async fn wait_for_status(&self, expeted_status: Status) {
        let mut status_rx = self.status_tx.subscribe();

        // SAFETY : We are borrowing self, constructor and sole owner of the sender here above. Hence it always exists
        #[allow(clippy::unwrap_used)]
        status_rx
            .wait_for(|status| *status == expeted_status)
            .await
            .unwrap();
    }

    fn task(&self) -> CredentialRequestTask {
        CredentialRequestTask::new(
            self.account.clone(),
            self.device.clone(),
            self.pending_storage.clone(),
            self.vpn_api_client.clone(),
            self.cached_data.clone(),
        )
    }

    // The trait methods below wrap these with `run_while_active` so they honour pause/resume.

    async fn do_fetch_ticketbooks(
        &self,
        ticketbook_type: TicketType,
    ) -> Result<Vec<NymCredential>, VpnApiFetcherError> {
        let outcome = self
            .task()
            .request_zk_nym_ticketbook(ticketbook_type)
            .await?;
        Ok(vec![outcome])
    }

    async fn do_fetch_master_verification_key(
        &self,
        epoch_id: EpochId,
    ) -> Result<EpochVerificationKey, VpnApiFetcherError> {
        let key = self
            .cached_data
            .get_master_verification_key(epoch_id)
            .await?;
        Ok(EpochVerificationKey { epoch_id, key })
    }

    async fn do_fetch_coin_index_signatures(
        &self,
        epoch_id: EpochId,
    ) -> Result<AggregatedCoinIndicesSignatures, VpnApiFetcherError> {
        let response = self
            .vpn_api_client
            .get_directory_zk_nyms_ticketbook_aggregated_coin_indices_signatures(epoch_id)
            .await
            .map_err(VpnApiFetcherError::vpn_api_error(
                "get_directory_zk_nyms_ticketbook_aggregated_coin_indices_signatures",
            ))?;
        Ok(response.signatures)
    }

    async fn do_fetch_expiration_date_signatures(
        &self,
        expiration_date: Date,
        epoch_id: EpochId,
    ) -> Result<AggregatedExpirationDateSignatures, VpnApiFetcherError> {
        let response = self
            .vpn_api_client
            .get_directory_zk_nyms_ticketbook_aggregated_expiration_date_signatures(
                epoch_id,
                expiration_date,
            )
            .await
            .map_err(VpnApiFetcherError::vpn_api_error(
                "get_directory_zk_nyms_ticketbook_aggregated_expiration_date_signatures",
            ))?;
        Ok(response.signatures)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl CredentialPublicDataFetcher for VpnApiCredentialFetcher {
    async fn fetch_master_verification_key(
        &self,
        epoch_id: EpochId,
    ) -> Result<EpochVerificationKey, CredentialFetcherError> {
        Ok(self
            .run_while_active(|| {
                with_retries(|| self.do_fetch_master_verification_key(epoch_id))
            })
            .await??)
    }

    async fn fetch_coin_index_signatures(
        &self,
        epoch_id: EpochId,
    ) -> Result<AggregatedCoinIndicesSignatures, CredentialFetcherError> {
        Ok(self
            .run_while_active(|| {
                with_retries(|| self.do_fetch_coin_index_signatures(epoch_id))
            })
            .await??)
    }

    async fn fetch_expiration_date_signatures(
        &self,
        expiration_date: Date,
        epoch_id: EpochId,
    ) -> Result<AggregatedExpirationDateSignatures, CredentialFetcherError> {
        Ok(self
            .run_while_active(|| {
                with_retries(|| {
                    self.do_fetch_expiration_date_signatures(expiration_date, epoch_id)
                })
            })
            .await??)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl CredentialFetcher for VpnApiCredentialFetcher {
    async fn fetch_ticketbooks(
        &self,
        ticketbook_type: TicketType,
    ) -> Result<Vec<NymCredential>, CredentialFetcherError> {
        // Cleaning up stale requests as a tidy task. Calling this here out of convenience
        self.pending_storage
            .clean_up_stale_requests()
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to clean up stale requests: {:?}", err);
            })
            .ok();
        Ok(self
            .run_while_active(|| {
                with_retries(|| self.do_fetch_ticketbooks(ticketbook_type))
            })
            .await??)
    }

    async fn cleanup(&self) {
        self.cancellation_token.cancel();
        self.pending_storage.close().await;
    }

    async fn reset(self) -> Result<(), CredentialFetcherError> {
        self.cancellation_token.cancel();
        Ok(self
            .pending_storage
            .reset()
            .await
            .map_err(VpnApiFetcherError::from)?)
    }
}
