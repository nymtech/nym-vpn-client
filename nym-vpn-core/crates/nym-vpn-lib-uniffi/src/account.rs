// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, sync::Arc, time::Duration};

use tokio::{sync::Mutex, task::JoinHandle};
use tokio_util::sync::{CancellationToken, DropGuard};

use nym_vpn_account_controller::{AccountCommandSender, AccountStateReceiver, NyxdClient};
use nym_vpn_api_client::types::Platform;
use nym_vpn_lib::storage::VpnClientOnDiskStorage;
use nym_vpn_lib_types::{
    AccountControllerState, DeeplinkClient, DeeplinkKind, GetDeeplinkParams,
    RegisterAccountRequest, RegisterAccountResponse, StoreAccountRequest, UserAgent,
    VpnAccountSummary,
};

use crate::{environment::NymEnvironment, error::VpnError, offline_monitor::NymOfflineMonitor};

struct State {
    join_handle: JoinHandle<()>,
    shutdown_drop_guard: DropGuard,
}

#[derive(uniffi::Object)]
pub struct NymAccountController {
    command_sender: AccountCommandSender,
    state_receiver: AccountStateReceiver,
    network_env: Arc<NymEnvironment>,
    state: Arc<Mutex<Option<State>>>,
}

#[uniffi::export(async_runtime = "tokio")]
impl NymAccountController {
    #[uniffi::constructor]
    pub async fn new(
        data_dir: PathBuf,
        user_agent: UserAgent,
        network_env: Arc<NymEnvironment>,
        offline_monitor: Arc<NymOfflineMonitor>,
    ) -> Result<Self, VpnError> {
        let storage = VpnClientOnDiskStorage::new(data_dir.clone());
        let shutdown_token = CancellationToken::new();

        let nym_vpn_api_client = nym_vpn_api_client::VpnApiClient::from_network(
            network_env.inner().nym_network_details(),
            Some(user_agent.into()),
            None,
        )
        .await
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })?;

        let nyxd_client = NyxdClient::new(network_env.inner());
        let account_controller_config = nym_vpn_account_controller::AccountControllerConfig {
            data_dir,
            network_env: network_env.inner().clone(),
        };

        let connectivity_handle = offline_monitor.inner();
        let account_controller = nym_vpn_account_controller::AccountController::new(
            nym_vpn_api_client,
            nyxd_client,
            account_controller_config,
            storage,
            connectivity_handle,
            shutdown_token.child_token(),
        )
        .await
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })?;

        let command_sender = account_controller.get_command_sender();
        let state_receiver = account_controller.get_state_receiver();
        let join_handle = tokio::spawn(account_controller.run());
        let shutdown_drop_guard = shutdown_token.drop_guard();

        Ok(Self {
            command_sender,
            state_receiver,
            network_env,
            state: Arc::new(Mutex::new(Some(State {
                join_handle,
                shutdown_drop_guard,
            }))),
        })
    }

    pub async fn shutdown_and_wait(&self) {
        let Some(state) = self.state.lock().await.take() else {
            return;
        };

        drop(state.shutdown_drop_guard);
        if let Err(err) = state.join_handle.await {
            tracing::error!("Failed to wait on account controller join handle: {err}");
        }
    }

    pub async fn wait_for_account_ready_to_connect(
        &self,
        timeout: Duration,
    ) -> Result<(), VpnError> {
        let mut cloned_receiver = self.state_receiver.clone();
        tokio::time::timeout(timeout, cloned_receiver.wait_for_account_ready_to_connect())
            .await
            .map_err(|_| VpnError::VpnApiTimeout)?
            .map_err(VpnError::from)
    }

    pub async fn get_deeplink(&self, params: GetDeeplinkParams) -> Result<String, VpnError> {
        let base_url = match params.kind {
            DeeplinkKind::Privy => {
                let Some(ref account_management) =
                    self.network_env.inner().nym_vpn_network.account_management
                else {
                    return Err(VpnError::DeeplinkError {
                        details: "No account management data is available at this time".to_owned(),
                    });
                };

                let opt_url = match params.client {
                    DeeplinkClient::Mobile => account_management.privy_mobile_url(&params.locale),
                    DeeplinkClient::Desktop => account_management.privy_desktop_url(&params.locale),
                    DeeplinkClient::Web => account_management.privy_web_url(&params.locale),
                };

                opt_url.ok_or(VpnError::DeeplinkError {
                    details: "The privy path could not be determined".to_owned(),
                })?
            }
        };

        self.command_sender
            .get_deeplink(params.kind, params.name, base_url)
            .await
            .map_err(VpnError::from)
    }

    pub async fn login_with_deeplink(&self, deeplink_callback_url: String) -> Result<(), VpnError> {
        let deeplink_mnemonic = self
            .command_sender
            .derive_deeplink_mnemonic(deeplink_callback_url)
            .await?;

        let privy_account = StorableAccount {
            mnemonic: deeplink_mnemonic.mnemonic.clone(),
            mode: StorableAccountMode::Privy,
        };

        match deeplink_mnemonic.kind {
            DeeplinkKind::Privy => self
                .command_sender
                .store_account(privy_account)
                .await
                .map_err(VpnError::from),
            DeeplinkKind::PrivyLink => self
                .command_sender
                .link_account(privy_account)
                .await
                .map_err(VpnError::from),
        }
    }

    pub async fn get_account_summary(&self) -> Result<Option<VpnAccountSummary>, VpnError> {
        self.command_sender
            .get_account_summary()
            .await
            .map_err(VpnError::from)
    }

    /// Get the account state
    pub async fn get_account_state(&self) -> AccountControllerState {
        self.state_receiver.get_state()
    }

    /// This manually syncs the account state with the server. Normally this is done automatically, but
    /// this can be used to manually trigger a sync.
    pub async fn update_account_state(&self) -> Result<(), VpnError> {
        self.command_sender
            .background_refresh_account_state()
            .await
            .map_err(VpnError::from)
    }

    /// Import the account mnemonic
    pub async fn login(&self, request: &StoreAccountRequest) -> Result<(), VpnError> {
        let mnemonic = nym_vpn_lib::login::parse_account_request(request).map_err(|err| {
            VpnError::InvalidSecret {
                details: err.to_string(),
            }
        })?;
        self.command_sender.store_account(mnemonic.into()).await?;
        Ok(())
    }

    /// Generate the account mnemonic locally and store it.
    pub async fn create_account(&self) -> Result<(), VpnError> {
        self.command_sender
            .create_account_command()
            .await
            .map_err(VpnError::from)
    }

    /// Register the stored account.
    pub async fn register_account(
        &self,
        request: RegisterAccountRequest,
    ) -> Result<RegisterAccountResponse, VpnError> {
        let mnemonic = self
            .command_sender
            .get_stored_account()
            .await
            .map_err(VpnError::from)?
            .ok_or(VpnError::NoAccountStored)?;
        let platform = Platform::from(request);
        self.command_sender
            .register_account(mnemonic, platform)
            .await
            .map_err(VpnError::from)
    }

    /// Remove the account mnemonic and all associated keys and files
    pub async fn forget_account(&self) -> Result<(), VpnError> {
        self.command_sender
            .forget_account()
            .await
            .map_err(VpnError::from)
    }

    /// Force a rotation of the wireguard keys
    pub async fn rotate_keys(&self) -> Result<(), VpnError> {
        self.command_sender
            .rotate_keys()
            .await
            .map_err(VpnError::from)
    }

    /// Get the account identity
    pub async fn get_account_identity(&self) -> Result<Option<String>, VpnError> {
        Ok(self.command_sender.get_account_id().await?)
    }

    /// Check if the account mnemonic is stored
    pub async fn is_account_mnemonic_stored(&self) -> Result<bool, VpnError> {
        Ok(self.command_sender.get_account_id().await?.is_some())
    }

    /// Read and return the mnemonic, if there's one stored.
    pub async fn get_stored_mnemonic(&self) -> Result<String, VpnError> {
        Ok(self
            .command_sender
            .get_stored_account()
            .await
            .map_err(VpnError::from)?
            .ok_or(VpnError::NoAccountStored)?
            .mnemonic
            .to_string())
    }

    /// Get the device identity
    pub async fn get_device_identity(&self) -> Result<String, VpnError> {
        self.command_sender
            .get_device_identity()
            .await?
            .ok_or(VpnError::NoAccountStored)
    }
}
