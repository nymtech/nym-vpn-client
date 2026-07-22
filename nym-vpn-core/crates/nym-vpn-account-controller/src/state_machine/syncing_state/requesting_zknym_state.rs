// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_api_client::{
    VpnApiClient,
    error::FAIR_USAGE_DEPLETED_CODE_ID,
    types::{Device, VpnAccount},
};

use crate::{
    SharedAccountState,
    commands::{
        AccountCommand, UpgradeModeCommand, common_handler, handler,
        zknym_handler::RequestZkNymCommandHandler,
    },
    state_machine::{
        AccountControllerStateHandler, ErrorState, LoggedOutState, NextAccountControllerState,
        OfflineState, PrivateAccountControllerState, ReadyState, SyncMode, SyncingNetworkState,
        UpgradeModeState,
    },
    storage::VpnCredentialStorage,
};
use nym_vpn_lib_types::{
    AccountCommandError, AccountControllerErrorStateReason, RequestZkNymErrorReason,
    RequestZkNymSuccess,
};
use tokio::{
    sync::mpsc,
    task::{JoinError, JoinHandle},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

// The maximum number of zk-nym requests that can fail in a row
const ZK_NYM_MAX_FAILS: u32 = 10;
const ZK_NYM_STATE_CONTEXT: &str = "ZK_NYM_STATE";

/// RequestingZkNyms State
/// That state gets ZK-nyms if needed.
/// This is a substate of the Syncing State, it will disappear into the future Bandwidth Controller
///
/// A retry mechanism is in place, if the error is in the API requests. Other errors lead a single retry, then to the ErrorState since they are not recoverable.  
///
/// Possible next state :
/// - ReadyState : We have enough tickets already, or we managed to get enough tickets
/// - ErrorState : An error happened, preventing us to proceed.
/// - OfflineState : the connectivity monitor is telling we're not connected
/// - SyncingNetworkState : We handled a (force) refresh account command, or a join/retry
/// - LoggedOutState : A successful forget account command was handled
/// - UpgradeModeState : Instead of retrieving zk-nyms, we have received information about upgrade mode being activated
pub(crate) struct RequestingZkNymsState {
    zk_nym_fetching_handle: JoinHandle<Result<ZkNymFetchResult, ZkNymError>>,
    attempts: u32,
    entered_through_upgrade_mode: bool,
}

impl RequestingZkNymsState {
    pub(crate) fn enter<C: ConnectivityMonitor>(
        shared_state: &SharedAccountState<C>,
        attempts: u32,
        entered_through_upgrade_mode: bool,
    ) -> (
        Box<dyn AccountControllerStateHandler<C>>,
        PrivateAccountControllerState,
    ) {
        let Some(vpn_api_account) = shared_state.vpn_api_account.clone() else {
            return LoggedOutState::enter();
        };
        let Some(device) = shared_state.device.clone() else {
            return ErrorState::enter(AccountControllerErrorStateReason::Internal {
                context: ZK_NYM_STATE_CONTEXT.to_string(),
                details: "Logged in, but no device keys".into(),
            });
        };
        let Some(ref vpn_account_summary) = shared_state.vpn_account_summary else {
            return ErrorState::enter(AccountControllerErrorStateReason::Internal {
                context: ZK_NYM_STATE_CONTEXT.to_string(),
                details: "Logged in, but no account summary".into(),
            });
        };

        let vpn_api_client = shared_state.vpn_api_client.clone();
        let fair_usage_left = vpn_account_summary.fair_usage_left();

        // can we make that unique to that state?
        let storage = shared_state.credential_storage.clone();
        let zk_nym_fetching_handle = tokio::spawn(async move {
            RequestingZkNymsState::fetch_zk_nyms(
                vpn_api_client,
                vpn_api_account,
                device,
                storage,
                fair_usage_left,
            )
            .await
        });

        (
            Box::new(Self {
                zk_nym_fetching_handle,
                attempts,
                entered_through_upgrade_mode,
            }),
            PrivateAccountControllerState::RequestingZkNyms,
        )
    }
    async fn fetch_zk_nyms(
        vpn_api_client: VpnApiClient,
        vpn_api_account: Arc<VpnAccount>,
        device: Device,
        storage: VpnCredentialStorage,
        fair_usage_left: bool,
    ) -> Result<ZkNymFetchResult, ZkNymError> {
        if !fair_usage_left {
            return Err(ZkNymError::BandwidthExceeded);
        }

        let ticket_types_to_request = storage
            .get_ticket_types_running_low()
            .await
            .inspect_err(|e| error!("zk-nym storage error: {e}"))
            .map_err(|err| ZkNymError::Storage(err.to_string()))?;

        if ticket_types_to_request.is_empty() {
            // We have enough credential, we can return
            return Ok(ZkNymFetchResult::SufficientBandwidth);
        }

        let request_handler =
            RequestZkNymCommandHandler::new(vpn_api_account, device, storage, vpn_api_client);

        let mut fetched_tickets: Option<Vec<_>> = None;
        let mut upgrade_mode = false;
        for partial_result in request_handler
            .request_zk_nyms(ticket_types_to_request)
            .await
        {
            let success = match partial_result.map_err(RequestZkNymErrorReason::from) {
                Ok(success) => success,
                Err(err) => {
                    return match err {
                        RequestZkNymErrorReason::VpnApi(inner) => {
                            let code = inner.code_reference_id();
                            error!("something went wrong trying to request zk-nym: {inner}");
                            if let Some(code_id) = &code
                                && code_id == FAIR_USAGE_DEPLETED_CODE_ID
                            {
                                Err(ZkNymError::BandwidthExceeded)
                            } else {
                                Err(ZkNymError::ApiFailure(code.unwrap_or_default()))
                            }
                        }
                        RequestZkNymErrorReason::UnexpectedVpnApiResponse(inner) => {
                            error!("unexpected response trying to request zk-nym: {inner}");
                            Err(ZkNymError::ApiFailure(inner))
                        }

                        RequestZkNymErrorReason::Storage(e) => {
                            error!("storage error trying to request zk-nym: {e}");
                            Err(ZkNymError::Storage(e))
                        }
                        RequestZkNymErrorReason::Internal(e) => {
                            error!("internal error trying to request zk-nym: {e}");
                            Err(ZkNymError::Internal(e))
                        }
                    };
                }
            };

            let id = match success {
                RequestZkNymSuccess::Ticketbook {
                    ticketbook_type,
                    id,
                } => {
                    fetched_tickets
                        .get_or_insert_with(Vec::new)
                        .push(ticketbook_type);
                    id
                }
                RequestZkNymSuccess::UpgradeMode { id } => {
                    upgrade_mode = true;
                    id
                }
            };
            debug!("managed to resolve zk-nym request '{id}'");
        }

        match (fetched_tickets, upgrade_mode) {
            (Some(tickets), true) => {
                // edge case where upgrade mode has been triggered in between concurrent ticket requests
                // incredibly unlikely, but always fallback to the upgrade mode
                info!(
                    "we managed to fetch the following tickets just before upgrade mode has been triggered: {tickets:?}"
                );
                Ok(ZkNymFetchResult::UpgradeMode)
            }
            (Some(tickets), false) => {
                debug!("managed to retrieve ticketbooks of the following types: {tickets:?}");
                Ok(ZkNymFetchResult::FetchedTickets { types: tickets })
            }
            (None, true) => {
                info!(
                    "upgrade mode has been triggered while attempting to fetch additional zk-nym tickets"
                );
                Ok(ZkNymFetchResult::UpgradeMode)
            }
            (None, false) => {
                // this branch should be impossible, given that we did have more than one result
                // and on any errors, we would have returned.
                // however, it's better to return a nonsense error than crashing in case something changes in the impl in the future
                error!(
                    "BUG DETECTED: did not receive ticketbooks nor upgrade mode information after successful zk-nym request"
                );
                Err(ZkNymError::internal(
                    "did not receive ticketbooks nor upgrade mode information after successful zk-nym request",
                ))
            }
        }
    }

    async fn handle_retrieved_zk_nym<C: ConnectivityMonitor>(
        self: Box<Self>,
        shared_state: &mut SharedAccountState<C>,
        zknym_result: Result<Result<ZkNymFetchResult, ZkNymError>, JoinError>,
    ) -> NextAccountControllerState<C> {
        let join_result = match zknym_result {
            Ok(join_result) => join_result,
            Err(err) => {
                error!("Failed to join on the fetching task, task probably got cancelled : {err}");
                // `self.zk_nym_fetching_handle` has already resolved (that's how we got here),
                // so it must never be kept around via `SameState`: polling an already-completed
                // `JoinHandle` again panics. Enter a fresh `RequestingZkNymsState` (with its own
                // freshly spawned handle) instead, following the same retry-with-attempts
                // pattern used for `ZkNymError` below.
                return if self.attempts > ZK_NYM_MAX_FAILS {
                    NextAccountControllerState::NewState(ErrorState::enter(
                        AccountControllerErrorStateReason::Internal {
                            context: ZK_NYM_STATE_CONTEXT.to_string(),
                            details: format!("zk-nym fetching task failed to join: {err}"),
                        },
                    ))
                } else {
                    NextAccountControllerState::NewState(RequestingZkNymsState::enter(
                        shared_state,
                        self.attempts + 1,
                        false,
                    ))
                };
            }
        };

        let retrieval_result = match join_result {
            Ok(retrieval_result) => retrieval_result,
            Err(zk_nym_error) => {
                if self.attempts > ZK_NYM_MAX_FAILS {
                    return NextAccountControllerState::NewState(ErrorState::enter(
                        zk_nym_error.into(),
                    ));
                }

                // We have an error, but maybe we still have enough ticketbook to proceed
                if let Ok(true) = shared_state
                    .credential_storage
                    .is_all_ticket_types_above_minimal_threshold()
                    .await
                {
                    // let's see if next sync fixes it
                    return NextAccountControllerState::NewState(ReadyState::enter());
                }
                return match zk_nym_error {
                    ZkNymError::Storage(_) | ZkNymError::Internal(_) => {
                        // Error is on our side, let's give it one last try though
                        NextAccountControllerState::NewState(RequestingZkNymsState::enter(
                            shared_state,
                            ZK_NYM_MAX_FAILS + 1,
                            false,
                        ))
                    }
                    ZkNymError::ApiFailure(_) => {
                        // Error on the API side, let's try again
                        NextAccountControllerState::NewState(RequestingZkNymsState::enter(
                            shared_state,
                            self.attempts + 1,
                            false,
                        ))
                    }
                    ZkNymError::BandwidthExceeded => {
                        tracing::warn!("Our fair usage is depleted");
                        if let Ok(true) = shared_state
                            .credential_storage
                            .is_all_ticket_types_non_empty()
                            .await
                        {
                            tracing::warn!("We still have some tickets though");
                            NextAccountControllerState::NewState(ReadyState::enter())
                        } else {
                            NextAccountControllerState::NewState(ErrorState::enter(
                                ZkNymError::BandwidthExceeded.into(),
                            ))
                        }
                    }
                };
            }
        };

        match retrieval_result {
            ZkNymFetchResult::SufficientBandwidth | ZkNymFetchResult::FetchedTickets { .. } => {
                NextAccountControllerState::NewState(ReadyState::enter())
            }
            ZkNymFetchResult::UpgradeMode => {
                NextAccountControllerState::NewState(UpgradeModeState::enter(shared_state).await)
            }
        }
    }

    async fn handle_account_command<C: ConnectivityMonitor>(
        self: Box<Self>,
        command: AccountCommand,
        shared_state: &mut SharedAccountState<C>,
    ) -> NextAccountControllerState<C> {
        match command {
            AccountCommand::CreateAccount(return_sender) => {
                return_sender.send(Err(AccountCommandError::ExistingAccount))
            }
            AccountCommand::StoreAccount(return_sender, _) => {
                return_sender.send(Err(AccountCommandError::ExistingAccount))
            }
            AccountCommand::RegisterAccount(return_sender, account, platform) => {
                let res = handler::handle_register_account(shared_state, account, platform).await;
                return_sender.send(res);
            }
            AccountCommand::ForgetAccount(return_sender) => {
                self.zk_nym_fetching_handle.abort();
                let res = handler::handle_forget_account(shared_state).await;
                let error = res.is_err();
                return_sender.send(res);
                return if error {
                    NextAccountControllerState::NewState(SyncingNetworkState::enter(
                        shared_state,
                        SyncMode::Optimistic,
                    ))
                } else {
                    NextAccountControllerState::NewState(LoggedOutState::enter())
                };
            }
            AccountCommand::LinkAccount(return_sender, privy_account) => {
                let res = handler::handle_link_account(shared_state, privy_account).await;
                return_sender.send(res);
            }
            AccountCommand::RotateKeys(return_sender) => {
                let res = handler::handle_rotate_keys(shared_state).await;
                return_sender.send(res);
            }
            AccountCommand::AccountBalance(return_sender) => {
                return_sender.send(Err(AccountCommandError::AccountNotDecentralised))
            }
            AccountCommand::ObtainTicketbooks(return_sender, _) => {
                return_sender.send(Err(AccountCommandError::AccountNotDecentralised))
            }
            AccountCommand::ResetDeviceIdentity(return_sender, seed) => {
                self.zk_nym_fetching_handle.abort();
                return_sender.send(handler::handle_reset_device_identity(shared_state, seed).await);
                return NextAccountControllerState::NewState(SyncingNetworkState::enter(
                    shared_state,
                    SyncMode::Mandatory,
                ));
            }
            AccountCommand::RefreshAccountState(return_sender, force) => {
                return_sender.send(Ok(()));
                return if shared_state.firewall_active {
                    NextAccountControllerState::SameState(self)
                } else {
                    self.zk_nym_fetching_handle.abort();
                    if force {
                        shared_state.mark_summary_as_stale();
                        return NextAccountControllerState::NewState(SyncingNetworkState::enter(
                            shared_state,
                            SyncMode::Mandatory,
                        ));
                    } else {
                        return NextAccountControllerState::NewState(SyncingNetworkState::enter(
                            shared_state,
                            SyncMode::Optimistic,
                        ));
                    }
                };
            }
            AccountCommand::VpnApiFirewallDown(return_sender) => {
                // as a side note, firewall handling could use some improvements as well,
                // because the current solution is suboptimal to say the least
                // and is causing a number of weird edge cases
                return_sender.send(Ok(()));
                // No-op if firewall was already down
                if shared_state.firewall_active {
                    // If firewall is indeed active, no handle should be running, so nothing to abort
                    shared_state.firewall_active = false;
                    return NextAccountControllerState::NewState(RequestingZkNymsState::enter(
                        shared_state,
                        self.attempts,
                        false,
                    ));
                }
            }
            AccountCommand::VpnApiFirewallUp(return_sender) => {
                shared_state.firewall_active = true;
                self.zk_nym_fetching_handle.abort();
                return_sender.send(Ok(()));
            }
            AccountCommand::Common(common_command) => {
                common_handler::handle_common_command(common_command, shared_state).await
            }
            AccountCommand::UpgradeMode(upgrade_mode_command) => match upgrade_mode_command {
                UpgradeModeCommand::GetUpgradeModeEnabled(return_sender) => {
                    return_sender.send(Ok(self.entered_through_upgrade_mode))
                }
                UpgradeModeCommand::DisableUpgradeMode(return_sender) => {
                    if !self.entered_through_upgrade_mode {
                        warn!(
                            "received unexpected command to disable upgrade mode while in 'RequestingZkNymsState' state"
                        );
                    }
                    // if UM is indeed over, the `zk_nym_fetching_handle` should resolve with
                    // fetched zk-nyms and we'll exit the state naturally.
                    // there's no need to abort the future

                    return_sender.send(Ok(()))
                }
            },
        }
        NextAccountControllerState::SameState(self)
    }
}

#[async_trait::async_trait]
impl<C: ConnectivityMonitor> AccountControllerStateHandler<C> for RequestingZkNymsState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<AccountCommand>,
        shared_state: &'async_trait mut SharedAccountState<C>,
    ) -> NextAccountControllerState<C> {
        tokio::select! {
            biased;
            _ = shutdown_token.cancelled() => {
                self.zk_nym_fetching_handle.abort();
                NextAccountControllerState::Finished
            }
            fetching_result = &mut self.zk_nym_fetching_handle => self.handle_retrieved_zk_nym(shared_state, fetching_result).await,
            Some(command) = command_rx.recv() => self.handle_account_command(command, shared_state).await,
            Some(connectivity) = shared_state.connectivity_handle.next() => {
                if connectivity.is_offline() {
                    self.zk_nym_fetching_handle.abort();
                    NextAccountControllerState::NewState(OfflineState::enter())
                } else {
                    NextAccountControllerState::SameState(self)
                }
            }
        }
    }
}

#[derive(Debug)]
enum ZkNymError {
    Storage(String),
    ApiFailure(String),
    Internal(String),
    BandwidthExceeded,
}

impl ZkNymError {
    fn internal<S: Into<String>>(msg: S) -> ZkNymError {
        ZkNymError::Internal(msg.into())
    }
}

impl From<ZkNymError> for AccountControllerErrorStateReason {
    fn from(value: ZkNymError) -> Self {
        use ZkNymError::*;
        match value {
            Storage(details) => Self::Storage {
                context: ZK_NYM_STATE_CONTEXT.to_string(),
                details,
            },
            ApiFailure(details) => Self::ApiFailure {
                context: ZK_NYM_STATE_CONTEXT.to_string(),
                details,
            },
            Internal(details) => Self::Internal {
                context: ZK_NYM_STATE_CONTEXT.to_string(),
                details,
            },
            BandwidthExceeded => Self::BandwidthExceeded {
                context: ZK_NYM_STATE_CONTEXT.to_string(),
            },
        }
    }
}

enum ZkNymFetchResult {
    SufficientBandwidth,
    FetchedTickets {
        #[allow(unused)]
        types: Vec<String>,
    },
    UpgradeMode,
}
