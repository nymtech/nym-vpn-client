// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::types::{Device, VpnApiAccount};
use nym_vpn_lib_types::RequestZkNymError;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{
    SharedAccountState,
    commands::{
        AccountCommand, common_handler, handler,
        zknym_handler::{RequestZkNymCommandHandler, RequestZkNymSummary},
    },
    state_machine::{
        AccountControllerStateHandler, ErrorState, LoggedOutState, NextAccountControllerState,
        OfflineState, PrivateAccountControllerState, ReadyState, SyncingState,
    },
    storage::VpnCredentialStorage,
    vpn_api_client::AccountControllerVpnApiClient,
};

pub(super) struct RequestingZkNymsState {
    zk_nym_fetching_handle: JoinHandle<Result<RequestZkNymSummary, RequestZkNymError>>,
}

impl RequestingZkNymsState {
    pub(super) async fn enter(
        shared_state: &SharedAccountState,
    ) -> (
        Box<dyn AccountControllerStateHandler>,
        PrivateAccountControllerState,
    ) {
        #[allow(clippy::unwrap_used)]
        let vpn_api_account = shared_state.vpn_api_account.clone().unwrap();
        #[allow(clippy::unwrap_used)]
        let device: Device = shared_state.device.clone().unwrap();
        let vpn_api_client = shared_state.vpn_api_client.clone();
        let storage = shared_state.credential_storage.clone();
        let zk_nym_fetching_handle = tokio::spawn(async move {
            RequestingZkNymsState::fetch_zk_nyms(vpn_api_client, vpn_api_account, device, storage)
                .await
        });

        (
            Box::new(Self {
                zk_nym_fetching_handle,
            }),
            PrivateAccountControllerState::RequestingZkNyms,
        )
    }
    async fn fetch_zk_nyms(
        vpn_api_client: AccountControllerVpnApiClient,
        vpn_api_account: VpnApiAccount,
        device: Device,
        storage: VpnCredentialStorage,
    ) -> Result<RequestZkNymSummary, RequestZkNymError> {
        #[allow(clippy::unwrap_used)]
        //SW of course we don't, but let's do that later, also, condition that on the credential_enabled flag
        if !storage
            .is_all_ticket_types_above_soft_threshold()
            .await
            .unwrap()
        {
            let test = RequestZkNymCommandHandler::new(
                vpn_api_account,
                device,
                storage,
                vpn_api_client.inner().clone(),
            );
            test.run().await
        } else {
            Ok(Vec::new())
        }
    }
}

#[async_trait::async_trait]
impl AccountControllerStateHandler for RequestingZkNymsState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<AccountCommand>,
        shared_state: &'async_trait mut SharedAccountState,
    ) -> NextAccountControllerState {
        tokio::select! {
            zknym_result = &mut self.zk_nym_fetching_handle => {
                match zknym_result {
                    Ok(result) => {
                        match result {
                            // SW better error handling
                            Ok(_) => { NextAccountControllerState::NewState(ReadyState::enter())},
                            Err(_) => {NextAccountControllerState::NewState(ErrorState::enter("idk"))},
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to join on the fetching task : {e}");
                        NextAccountControllerState::NewState(SyncingState::enter(shared_state))
                    }
                }
            },
        Some(command) = command_rx.recv() => {
                match command {
                    AccountCommand::CreateAccount(_) => {},
                    AccountCommand::StoreAccount(_, _) => {},
                    AccountCommand::RegisterAccount(_, _, _) => {},
                    AccountCommand::ForgetAccount(return_sender) => {
                        let res = handler::handle_forget_account(shared_state).await;
                        let error = res.is_err();
                        return_sender.send(res);
                        if error {
                            return NextAccountControllerState::NewState(SyncingState::enter(shared_state)); // SW we might be in an intermediate state here, double check that
                        } else {
                            return NextAccountControllerState::NewState(LoggedOutState::enter());
                        }
                    },
                    AccountCommand::ResetDeviceIdentity(return_sender, seed) => {
                        return_sender.send(handler::handle_reset_device_identity(shared_state, seed).await);
                        return NextAccountControllerState::NewState(SyncingState::enter(shared_state));
                    },
                    AccountCommand::RefreshAccountState(return_sender) => {
                        self.zk_nym_fetching_handle.abort();
                        return_sender.send(Ok(()));
                        return NextAccountControllerState::NewState(SyncingState::enter(shared_state));
                    },
                    AccountCommand::Common(common_command) => {
                        common_handler::handle_common_command(common_command, shared_state).await
                    },
                }
                NextAccountControllerState::SameState(self)
            }
            Some(connectivity) = shared_state.connectivity_handle.next() => {
                if connectivity.is_offline() {
                    NextAccountControllerState::NewState(OfflineState::enter())
                } else {
                    NextAccountControllerState::SameState(self)
                }
            }
            _ = shutdown_token.cancelled() => {
                NextAccountControllerState::Finished
            }
        }
    }
}
