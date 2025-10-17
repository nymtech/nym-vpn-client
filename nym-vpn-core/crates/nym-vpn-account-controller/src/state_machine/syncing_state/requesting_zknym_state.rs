// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_api_client::{
    VpnApiClient,
    error::FAIR_USAGE_DEPLETED_CODE_ID,
    types::{Device, VpnAccount},
};

use nym_vpn_lib_types::{
    AccountCommandError, AccountControllerErrorStateReason, RequestZkNymErrorReason,
};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{
    SharedAccountState,
    commands::{
        AccountCommand, common_handler, handler, zknym_handler::RequestZkNymCommandHandler,
    },
    state_machine::{
        AccountControllerStateHandler, ErrorState, LoggedOutState, NextAccountControllerState,
        OfflineState, PrivateAccountControllerState, ReadyState, SyncingState,
    },
    storage::VpnCredentialStorage,
};

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
/// - SyncingState : We handled a refresh account command
/// - LoggedOutState : A successful forget account command was handled
pub(super) struct RequestingZkNymsState {
    zk_nym_fetching_handle: JoinHandle<Result<(), ZkNymError>>,
    attempts: u32,
    fair_usage_left: bool,
}

impl RequestingZkNymsState {
    pub(super) fn enter<C: ConnectivityMonitor>(
        shared_state: &SharedAccountState<C>,
        attempts: u32,
        fair_usage_left: bool, // Syncing state telling us the fair usage state
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

        let vpn_api_client = shared_state.vpn_api_client.clone();

        // can we make that unique to that state?
        let storage = shared_state.credential_storage.clone();
        let credential_mode = shared_state.config.credentials_mode();
        let zk_nym_fetching_handle = tokio::spawn(async move {
            RequestingZkNymsState::fetch_zk_nyms(
                vpn_api_client,
                vpn_api_account,
                device,
                storage,
                credential_mode,
                fair_usage_left,
            )
            .await
        });

        (
            Box::new(Self {
                zk_nym_fetching_handle,
                attempts,
                fair_usage_left,
            }),
            PrivateAccountControllerState::RequestingZkNyms,
        )
    }
    async fn fetch_zk_nyms(
        vpn_api_client: VpnApiClient,
        vpn_api_account: VpnAccount,
        device: Device,
        storage: VpnCredentialStorage,
        credential_mode: bool,
        fair_usage_left: bool,
    ) -> Result<(), ZkNymError> {
        if !credential_mode {
            return Ok(());
        }

        if !fair_usage_left {
            return Err(ZkNymError::BandwidthExceeded);
        }

        let ticket_types_to_request = storage
            .get_ticket_types_running_low()
            .await
            .inspect_err(|e| tracing::error!("Zk-nym storage error : {e}"))
            .map_err(|_| ZkNymError::Storage)?;

        if ticket_types_to_request.is_empty() {
            // We have enough credential, we can return
            return Ok(());
        }

        let request_handler = RequestZkNymCommandHandler::new(
            vpn_api_account,
            device,
            storage,
            vpn_api_client.clone(),
        );
        for partial_result in request_handler
            .request_zk_nyms(ticket_types_to_request)
            .await
        {
            if let Err(e) = partial_result {
                match RequestZkNymErrorReason::from(e) {
                    RequestZkNymErrorReason::VpnApi(inner) => {
                        tracing::error!("Something went wrong trying to request zk-nym : {inner}");
                        if let Some(code_id) = inner.code_reference_id()
                            && code_id == FAIR_USAGE_DEPLETED_CODE_ID
                        {
                            return Err(ZkNymError::BandwidthExceeded);
                        } else {
                            return Err(ZkNymError::ApiFailure(
                                inner.code_reference_id().unwrap_or_default(),
                            ));
                        }
                    }
                    RequestZkNymErrorReason::UnexpectedVpnApiResponse(inner) => {
                        tracing::error!("Unexpected response trying to request zk-nym : {inner}");
                        return Err(ZkNymError::ApiFailure(inner));
                    }

                    RequestZkNymErrorReason::Storage(e) => {
                        tracing::error!("Storage error trying to request zk-nym : {e}");
                        return Err(ZkNymError::Storage);
                    }
                    RequestZkNymErrorReason::Internal(e) => {
                        tracing::error!("Internal error trying to request zk-nym : {e}");
                        return Err(ZkNymError::Internal(e));
                    }
                }
            }
        }

        Ok(())
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
            zknym_result = &mut self.zk_nym_fetching_handle => {
                match zknym_result {
                    Ok(result) => {
                        match result {
                            Ok(()) => {
                                // No error whatsoever, we're ready to connect
                                NextAccountControllerState::NewState(ReadyState::enter())
                            },
                            Err(zk_nym_error) => {
                                if self.attempts > ZK_NYM_MAX_FAILS {
                                    return NextAccountControllerState::NewState(ErrorState::enter(zk_nym_error.into()));
                                }

                                // We have an error, but maybe we still have enough ticketbook to proceed
                                if let Ok(true) = shared_state.credential_storage.is_all_ticket_types_above_minimal_threshold().await {
                                    // let's see if next sync fixes it
                                    return NextAccountControllerState::NewState(ReadyState::enter());
                                }
                                match zk_nym_error {
                                    ZkNymError::Storage | ZkNymError::Internal(_) => {
                                        // Error is on our side, let's give it one last try though
                                        NextAccountControllerState::NewState(RequestingZkNymsState::enter(shared_state, ZK_NYM_MAX_FAILS + 1, self.fair_usage_left))
                                    },
                                    ZkNymError::ApiFailure(_) => {
                                        // Error on the API side, let's try again
                                        NextAccountControllerState::NewState(RequestingZkNymsState::enter(shared_state, self.attempts + 1, self.fair_usage_left))
                                    },
                                    ZkNymError::BandwidthExceeded => {
                                        tracing::warn!("Our fair usage is depleted");
                                        if let Ok(true) = shared_state.credential_storage.is_all_ticket_types_non_empty().await {
                                            tracing::warn!("We still have some tickets though");
                                            NextAccountControllerState::NewState(ReadyState::enter())
                                        } else {
                                            NextAccountControllerState::NewState(ErrorState::enter(ZkNymError::BandwidthExceeded.into()))
                                        }
                                    },
                                }
                            },
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to join on the fetching task : {e}");
                        NextAccountControllerState::NewState(SyncingState::enter(shared_state, self.attempts + 1))
                    }
                }
            },
        Some(command) = command_rx.recv() => {
                match command {
                    AccountCommand::CreateAccount(return_sender) => return_sender.send(Err(AccountCommandError::ExistingAccount)),
                    AccountCommand::StoreAccount(return_sender, _) => return_sender.send(Err(AccountCommandError::ExistingAccount)),
                    AccountCommand::RegisterAccount(return_sender, account, platform) => {
                        let res = handler::handle_register_account(shared_state, account, platform).await;
                        return_sender.send(res);
                    }                    AccountCommand::ForgetAccount(return_sender) => {
                        self.zk_nym_fetching_handle.abort();
                        let res = handler::handle_forget_account(shared_state).await;
                        let error = res.is_err();
                        return_sender.send(res);
                        if error {
                            return NextAccountControllerState::NewState(SyncingState::enter(shared_state, 0));
                        } else {
                            return NextAccountControllerState::NewState(LoggedOutState::enter());
                        }
                    },
                        AccountCommand::AccountBalance(return_sender) => return_sender.send(Err(AccountCommandError::AccountNotDecentralised)),
                    AccountCommand::ObtainTicketbooks(return_sender, _) => return_sender.send(Err(AccountCommandError::AccountNotDecentralised)),
                    AccountCommand::ResetDeviceIdentity(return_sender, seed) => {
                        self.zk_nym_fetching_handle.abort();
                        return_sender.send(handler::handle_reset_device_identity(shared_state, seed).await);
                        return NextAccountControllerState::NewState(SyncingState::enter(shared_state, 0));
                    },
                    AccountCommand::RefreshAccountState(return_sender) => {
                        return_sender.send(Ok(()));
                        if shared_state.firewall_active {
                            return NextAccountControllerState::SameState(self);
                        } else {
                            self.zk_nym_fetching_handle.abort();
                            return NextAccountControllerState::NewState(SyncingState::enter(shared_state, 0));
                        }
                    },

                    AccountCommand::VpnApiFirewallDown(return_sender) =>  {
                        shared_state.firewall_active = false;
                        return_sender.send(Ok(()));
                        return NextAccountControllerState::NewState(RequestingZkNymsState::enter(shared_state, self.attempts, self.fair_usage_left));
                    },

                    AccountCommand::VpnApiFirewallUp(return_sender) => {
                        shared_state.firewall_active = true;
                        self.zk_nym_fetching_handle.abort();
                        return_sender.send(Ok(()));
                    },
                    AccountCommand::Common(common_command) => {
                        common_handler::handle_common_command(common_command, shared_state).await
                    },
                }
                NextAccountControllerState::SameState(self)
            }
            Some(connectivity) = shared_state.connectivity_handle.next() => {
                if connectivity.is_offline() {
                    self.zk_nym_fetching_handle.abort();
                    NextAccountControllerState::NewState(OfflineState::enter())
                } else {
                    NextAccountControllerState::SameState(self)
                }
            }
            _ = shutdown_token.cancelled() => {
                self.zk_nym_fetching_handle.abort();
                NextAccountControllerState::Finished
            }
        }
    }
}

#[derive(Debug)]
enum ZkNymError {
    Storage,
    ApiFailure(String),
    Internal(String),
    BandwidthExceeded,
}

impl From<ZkNymError> for AccountControllerErrorStateReason {
    fn from(value: ZkNymError) -> Self {
        use ZkNymError::*;
        match value {
            Storage => Self::Storage {
                context: ZK_NYM_STATE_CONTEXT.to_string(),
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
