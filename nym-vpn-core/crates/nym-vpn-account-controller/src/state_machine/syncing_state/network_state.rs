// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{sync::Arc, time::Duration};

use crate::{
    SharedAccountState,
    commands::{AccountCommand, common_handler, handler},
    state_machine::{
        AccountControllerStateHandler, DecentralisedState, ErrorState, LoggedOutState,
        NextAccountControllerState, OfflineState, PrivateAccountControllerState,
        syncing_state::{SyncMode, local_state::SyncingLocalState},
    },
};
use futures::{FutureExt, future::Fuse};
use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_api_client::{
    VpnApiClient,
    error::VpnApiClientError,
    response::NymErrorResponse,
    types::{Device, VpnAccount},
};
use nym_vpn_lib_types::{
    AccountCommandError, AccountControllerErrorStateReason, VpnAccountSummary,
};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::{CancellationToken, DropGuard};

const SYNCING_NETWORK_STATE_CONTEXT: &str = "SYNCING_NETWORK_STATE";

/// In optimistic mode, how long we wait for the summary fetch before falling back to the cached
/// summary.
const OPTIMISTIC_SYNC_TIMEOUT: Duration = Duration::from_secs(5);

type SyncResult = Result<Option<VpnAccountSummary>, SyncError>;

/// SyncingNetwork State
/// This is the "network" half of syncing: it talks to the VPN API to fetch the account summary
/// (after making sure the device clock isn't too desynced, otherwise zk-nyms would fail to verify
/// on gateways) and stores it both in memory and on disk.
///
/// Once the summary is stored, control is handed over to [`SyncingLocalState`] which performs the
/// actual checks on it.
///
/// It operates in one of two [`SyncMode`]s:
/// - [`SyncMode::Optimistic`]: the fetch races a short timeout. If it doesn't complete in time or
///   fails for any reason, we fall back to the cached summary (one is guaranteed to exist - `enter`
///   downgrades to `Mandatory` when there is no cache) and run the checks on it.
/// - [`SyncMode::Mandatory`]: a full fetch with no cache fallback, used for forced/stale/retry
///   syncs. Failures retry with backoff and eventually land in the error state.
///
/// Possible next state :
/// - LoggedOutState : No account is stored
/// - SyncingLocalState : A summary is available (freshly fetched, or the cache after an optimistic
///   miss) and its checks can run. We always hand off with an existing non-stale summary.
/// - SyncingNetworkState : A mandatory fetch failed and we retry (with backoff)
/// - ErrorState : A mandatory fetch exhausted its retries, preventing us from proceeding
/// - OfflineState : the connectivity monitor is telling we're not connected
/// - DecentralisedState : The loaded account is set to "decentralised" mode
pub(crate) struct SyncingNetworkState {
    result_rx: Fuse<oneshot::Receiver<SyncResult>>,
    sync_cancel_token: Option<DropGuard>,
}

impl SyncingNetworkState {
    pub(crate) async fn enter<C: ConnectivityMonitor>(
        shared_state: &mut SharedAccountState<C>,
        sync_mode: SyncMode,
    ) -> (
        Box<dyn AccountControllerStateHandler<C>>,
        PrivateAccountControllerState,
    ) {
        let Some(vpn_api_account) = shared_state.vpn_api_account.clone() else {
            return LoggedOutState::enter(shared_state).await;
        };
        if vpn_api_account.mode().is_decentralised() {
            return DecentralisedState::enter();
        }
        let Some(device) = shared_state.device.clone() else {
            return ErrorState::enter(
                shared_state,
                AccountControllerErrorStateReason::Internal {
                    context: SYNCING_NETWORK_STATE_CONTEXT.into(),
                    details: "Logged in, but no device keys".into(),
                },
            )
            .await;
        };

        let vpn_api_client = shared_state.vpn_api_client.clone();

        // Use the provided sync mode unless there is no cache
        let sync_mode = if shared_state.vpn_account_summary.is_none() {
            SyncMode::Mandatory
        } else {
            sync_mode
        };
        tracing::debug!("Optimistic sync? : {}", sync_mode == SyncMode::Optimistic);

        let (result_tx, result_rx) = oneshot::channel();
        let sync_cancel_token = CancellationToken::new();

        // This handle does not need to be awaited since event channel and cancellation token are sufficient.
        let _syncing_state_handle =
            tokio::spawn(sync_cancel_token.child_token().run_until_cancelled_owned(
                SyncingNetworkState::sync_network(
                    result_tx,
                    vpn_api_client,
                    vpn_api_account,
                    device,
                    sync_mode,
                ),
            ));

        (
            Box::new(Self {
                result_rx: result_rx.fuse(),
                sync_cancel_token: Some(sync_cancel_token.drop_guard()),
            }),
            PrivateAccountControllerState::Syncing,
        )
    }

    async fn sync_network(
        result_tx: oneshot::Sender<SyncResult>,
        vpn_api_client: VpnApiClient,
        vpn_api_account: Arc<VpnAccount>,
        device: Device,
        sync_mode: SyncMode,
    ) {
        let fetch = Self::fetch_summary(&vpn_api_client, &vpn_api_account, &device);

        let sync_result = match sync_mode {
            SyncMode::Optimistic => {
                match tokio::time::timeout(OPTIMISTIC_SYNC_TIMEOUT, fetch).await {
                    Ok(Ok(summary)) => Ok(Some(summary)),
                    Ok(Err(err)) => {
                        // We are packing all errors here, but we don't expect a lot of errors that aren't API failure
                        // Hence it can wait a next forced sync
                        tracing::warn!(
                            "optimistic account summary sync ended with an error : {err}"
                        );
                        Ok(None)
                    }
                    Err(_elapsed) => {
                        tracing::warn!("optimistic account summary sync timed out");
                        Ok(None)
                    }
                }
            }
            SyncMode::Mandatory => fetch.await.map(Some),
        };
        result_tx.send(sync_result).ok();
    }

    async fn fetch_summary(
        vpn_api_client: &VpnApiClient,
        vpn_api_account: &VpnAccount,
        device: &Device,
    ) -> Result<VpnAccountSummary, SyncError> {
        // Fetch the remote time so the summary can record whether our clock is acceptably synced
        // (a desync would make zk-nyms fail to verify on gateways). The desync itself is surfaced
        // later, during the local checks, via `VpnAccountSummary::time_synced`.
        let remote_time = vpn_api_client.get_remote_time().await?;

        let summary = vpn_api_client
            .get_account_summary_with_device(vpn_api_account, device)
            .await?;

        tracing::debug!("{summary:#?}");

        let summary = VpnAccountSummary::from_parts(&summary, vpn_api_account.mode(), remote_time)
            .map_err(|err| SyncError::ApiResponseError {
                details: format!("Failed to create account summary from API response: {err}"),
            })?;

        Ok(summary)
    }
}

#[async_trait::async_trait]
impl<C: ConnectivityMonitor> AccountControllerStateHandler<C> for SyncingNetworkState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<AccountCommand>,
        shared_state: &'async_trait mut SharedAccountState<C>,
    ) -> NextAccountControllerState<C> {
        tokio::select! {
            biased;
            _ = shutdown_token.cancelled() => {
                NextAccountControllerState::Finished
            }
            Some(connectivity) = shared_state.connectivity_handle.next() => {
                if connectivity.is_offline() {
                    NextAccountControllerState::NewState(OfflineState::enter())
                } else {
                    NextAccountControllerState::SameState(self)
                }
            }
            sync_result = &mut self.result_rx => {
                match sync_result {
                    Ok(Ok(Some(response))) => {
                        // The summary is stored (and propagated to disk) even if the subsequent
                        // local checks eventually fail.
                        shared_state.store_summary(response);
                        NextAccountControllerState::NewState(SyncingLocalState::enter(shared_state).await)

                    }
                    Ok(Ok(None)) => {
                        // An optimistic refresh failed, no big deal
                        NextAccountControllerState::NewState(SyncingLocalState::enter(shared_state).await)
                    }
                    Ok(Err(err)) => {
                        // A mandatory sync failed
                        tracing::error!("Mandatory sync failed ({err})");
                        NextAccountControllerState::NewState(ErrorState::enter(shared_state, err.into_error_reason()).await)
                    }
                    Err(e) => {
                        tracing::error!("No result from network sync, task probably got cancelled : {e}");
                        NextAccountControllerState::SameState(self)
                    }
                }
            }
            Some(command) = command_rx.recv() => {
                match command {
                    AccountCommand::CreateAccount(return_sender) => return_sender.send(Err(AccountCommandError::ExistingAccount)),
                    AccountCommand::StoreAccount(return_sender, _) => return_sender.send(Err(AccountCommandError::ExistingAccount)),
                    AccountCommand::RegisterAccount(return_sender, account, platform) => {
                        let res = handler::handle_register_account(shared_state, account, platform).await;
                        return_sender.send(res);
                    }
                    AccountCommand::ForgetAccount(return_sender) => {
                        let res = handler::handle_forget_account(shared_state).await;
                        let error = res.is_err();
                        return_sender.send(res);
                        return if error {
                            NextAccountControllerState::SameState(self)
                        } else {
                            NextAccountControllerState::NewState(LoggedOutState::enter(shared_state).await)
                        }
                    },
                    AccountCommand::LinkAccount(return_sender, privy_account) => {
                        let res = handler::handle_link_account(shared_state, privy_account).await;
                        return_sender.send(res);
                    },
                    AccountCommand::RotateKeys(return_sender) => {
                        let res = handler::handle_rotate_keys(shared_state).await;
                        return_sender.send(res);
                    },
                    AccountCommand::AccountBalance(return_sender) => return_sender.send(Err(AccountCommandError::AccountNotDecentralised)),
                    AccountCommand::ObtainTicketbooks(return_sender) => return_sender.send(Err(AccountCommandError::AccountNotDecentralised)),
                    AccountCommand::RefreshAccountState(return_sender, force) => {
                        return_sender.send(Ok(()));
                        return if shared_state.firewall_active {
                            NextAccountControllerState::SameState(self)
                        } else {
                            if force {
                                shared_state.mark_summary_as_stale();
                                return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Mandatory).await);
                            } else {
                                return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Optimistic).await);
                            }
                        }
                    },
                    AccountCommand::ResetDeviceIdentity(return_sender, seed) => {
                        return_sender.send(handler::handle_reset_device_identity(shared_state, seed).await);
                        return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Mandatory).await);
                    },

                    AccountCommand::VpnApiFirewallDown(return_sender) =>  {
                        return_sender.send(Ok(()));
                        // No-op if the firewall was already down
                        if shared_state.set_firewall_state(false) {
                            return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Optimistic).await);
                        }
                    },

                    AccountCommand::VpnApiFirewallUp(return_sender) => {
                        if shared_state.set_firewall_state(true) {
                            // Explicitly cancel sync task since the same state persists.
                            // Sync will restart once firewall permits traffic to flow again.
                            self.sync_cancel_token.take();
                        }
                        return_sender.send(Ok(()));
                    },

                    AccountCommand::Common(common_command) => {
                        common_handler::handle_common_command(common_command, shared_state).await
                    },
                }
                NextAccountControllerState::SameState(self)
            }

        }
    }
}

#[derive(Debug, strum::Display)]
enum SyncError {
    ApiRequestError(String),
    ApiResponseError { details: String },
    UnregisteredAccount,
}

impl SyncError {
    /// Maps a network fetch failure to the reason carried into the error state.
    fn into_error_reason(self) -> AccountControllerErrorStateReason {
        use SyncError::*;
        match self {
            ApiRequestError(e) => AccountControllerErrorStateReason::ApiFailure {
                context: SYNCING_NETWORK_STATE_CONTEXT.into(),
                details: format!("Failure to reach the API {}", e),
            },
            ApiResponseError { details } => AccountControllerErrorStateReason::ApiFailure {
                context: SYNCING_NETWORK_STATE_CONTEXT.into(),
                details: format!("API returned an error: {}", details),
            },
            UnregisteredAccount => AccountControllerErrorStateReason::AccountStatusNotActive {
                status: "unregistered".into(),
            },
        }
    }
}

impl From<VpnApiClientError> for SyncError {
    fn from(value: VpnApiClientError) -> Self {
        match NymErrorResponse::try_from(value) {
            Ok(error_response) => {
                // SW Use UUID when it will be available
                if error_response.status == "access_denied"
                    && error_response.message == "Account not found"
                {
                    // Request was fine, but account is unregistered
                    // Later down the line we can maybe register it here
                    SyncError::UnregisteredAccount
                } else {
                    SyncError::ApiResponseError {
                        details: error_response
                            .code_reference_id
                            .unwrap_or(error_response.message),
                    }
                }
            }
            Err(e) => SyncError::ApiRequestError(e.to_string()),
        }
    }
}
