// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{environment::NymEnvironment, error::VpnError, offline_monitor::NymOfflineMonitor};

use std::{path::PathBuf, sync::Arc, time::Duration};

use nym_bandwidth_controller::{
    AvailableTicketbooks, BandwidthController, requests::BandwidthControllerRequestSender,
};
use tokio::{sync::Mutex, task::JoinHandle};
use tokio_util::sync::{CancellationToken, DropGuard};

use nym_vpn_account_controller::{AccountCommandSender, AccountStateReceiver, NyxdClient};
use nym_vpn_api_client::types::Platform;
use nym_vpn_lib::storage::VpnClientOnDiskStorage;
use nym_vpn_lib_types::{
    AccountControllerState, AutologinResponse, DeeplinkClient, DeeplinkKind, GetDeeplinkParams,
    RegisterAccountRequest, RegisterAccountResponse, StorableAccount, StoreAccountRequest,
    StoredAccountMode, UserAgent, VpnAccountSummary,
};

struct State {
    account_join_handle: JoinHandle<()>,
    bandwidth_join_handle: JoinHandle<()>,
    shutdown_drop_guard: DropGuard,
}

#[derive(uniffi::Object)]
pub struct NymAccountController {
    account_command_sender: AccountCommandSender,
    state_receiver: AccountStateReceiver,
    bandwidth_command_sender: BandwidthControllerRequestSender,
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
        // Storage setup
        let storage_path = data_dir.join(network_env.network_name());
        let storage = VpnClientOnDiskStorage::new(&storage_path);
        let sdk_storage_paths = nym_sdk::mixnet::StoragePaths::new_from_dir(&storage_path)
            .map_err(|err| VpnError::InternalError {
                details: err.to_string(),
            })?;
        let credential_storage = sdk_storage_paths
            .persistent_credential_storage()
            .await
            .map_err(|err| VpnError::InternalError {
                details: err.to_string(),
            })?;

        let shutdown_token = CancellationToken::new();

        // BC setup
        let bandwidth_controller = BandwidthController::new(credential_storage);
        let bandwidth_command_sender = bandwidth_controller.get_request_sender();
        let bandwidth_join_handle =
            tokio::spawn(bandwidth_controller.run(shutdown_token.child_token().into()));

        // AC setup
        let nym_vpn_api_client = nym_vpn_api_client::VpnApiClient::from_network(
            network_env.inner().nym_network_details(),
            Some(user_agent.into()),
        )
        .await
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })?;

        let nyxd_client = NyxdClient::new(network_env.inner());
        let account_controller_config = nym_vpn_account_controller::AccountControllerConfig {
            data_dir: storage_path,
            storage_paths: sdk_storage_paths,
            network_env: network_env.inner().clone(),
        };

        let connectivity_handle = offline_monitor.inner();
        let account_controller = nym_vpn_account_controller::AccountController::new(
            nym_vpn_api_client,
            nyxd_client,
            account_controller_config,
            storage,
            connectivity_handle,
            bandwidth_command_sender.clone(),
            shutdown_token.child_token(),
        )
        .await
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })?;

        let account_command_sender = account_controller.get_command_sender();
        let state_receiver = account_controller.get_state_receiver();
        let account_join_handle = tokio::spawn(account_controller.run());
        let shutdown_drop_guard = shutdown_token.drop_guard();

        Ok(Self {
            account_command_sender,
            bandwidth_command_sender,
            state_receiver,
            network_env,
            state: Arc::new(Mutex::new(Some(State {
                account_join_handle,
                bandwidth_join_handle,
                shutdown_drop_guard,
            }))),
        })
    }

    pub async fn shutdown_and_wait(&self) {
        let Some(state) = self.state.lock().await.take() else {
            return;
        };

        // No need for a particular ordering if they are shut down jointly
        drop(state.shutdown_drop_guard);
        if let Err(err) = state.account_join_handle.await {
            tracing::error!("Failed to wait on account controller join handle: {err}");
        }

        if let Err(err) = state.bandwidth_join_handle.await {
            tracing::error!("Failed to wait on bandwidth controller join handle: {err}");
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

    /// Wait until the bandwidth controller has stocked every required ticketbook type (or covered
    /// it via upgrade mode). Errors if a required type is neither stocked nor being fetched, which
    /// we treat as a failed prefetch. Must be called after the account is ready to connect, so the
    /// account controller has installed a credential fetcher on the bandwidth controller.
    pub async fn wait_for_ticketbooks(&self, timeout: Duration) -> Result<(), VpnError> {
        let ticketbook_types = AvailableTicketbooks::ticketbook_types();
        tokio::time::timeout(
            timeout,
            self.bandwidth_command_sender
                .wait_for_ticketbooks(ticketbook_types),
        )
        .await
        .map_err(|_| VpnError::VpnApiTimeout)?
        .map_err(|err| VpnError::ZkNymAcquisitionFailure {
            details: err.to_string(),
        })
    }

    pub async fn get_deeplink(&self, params: GetDeeplinkParams) -> Result<String, VpnError> {
        let Some(ref account_management) =
            self.network_env.inner().nym_vpn_network.account_management
        else {
            return Err(VpnError::DeeplinkError {
                details: "No account management data is available at this time".to_owned(),
            });
        };

        let base_url = match params.client {
            DeeplinkClient::Mobile => account_management.privy_mobile_url(&params.locale),
            DeeplinkClient::Desktop => account_management.privy_desktop_url(&params.locale),
            DeeplinkClient::Web => account_management.privy_web_url(&params.locale),
        }
        .ok_or(VpnError::DeeplinkError {
            details: "The privy path could not be determined".to_owned(),
        })?;

        self.account_command_sender
            .get_deeplink(params.kind, params.name, base_url)
            .await
            .map_err(VpnError::from)
    }

    pub async fn get_autologin_deeplink(
        &self,
        params: GetDeeplinkParams,
    ) -> Result<AutologinResponse, VpnError> {
        let Some(ref account_management) =
            self.network_env.inner().nym_vpn_network.account_management
        else {
            return Err(VpnError::DeeplinkError {
                details: "No account management data is available at this time".to_owned(),
            });
        };

        let base_url = match params.client {
            DeeplinkClient::Mobile => account_management.autologin_mobile_url(&params.locale),
            DeeplinkClient::Desktop => account_management.autologin_desktop_url(&params.locale),
            DeeplinkClient::Web => account_management.autologin_web_url(&params.locale),
        }
        .ok_or(VpnError::DeeplinkError {
            details: "The autologin path could not be determined".to_owned(),
        })?;

        self.account_command_sender
            .get_autologin_deeplink(params.kind, params.name, base_url)
            .await
            .map_err(VpnError::from)
    }

    pub async fn login_with_deeplink(&self, deeplink_callback_url: String) -> Result<(), VpnError> {
        let deeplink_mnemonic = self
            .account_command_sender
            .derive_deeplink_mnemonic(deeplink_callback_url)
            .await?;

        let privy_account = StorableAccount {
            mnemonic: deeplink_mnemonic.mnemonic.clone(),
            mode: StoredAccountMode::Privy,
        };

        match deeplink_mnemonic.kind {
            DeeplinkKind::Privy | DeeplinkKind::CreateAccount => self
                .account_command_sender
                .store_account(privy_account)
                .await
                .map_err(VpnError::from),
            DeeplinkKind::PrivyLink => self
                .account_command_sender
                .link_account(privy_account)
                .await
                .map_err(VpnError::from),
            DeeplinkKind::AutologinRenew | DeeplinkKind::AutologinView => {
                Err(VpnError::DeeplinkError {
                    details: "Invalid deeplink kind".to_owned(),
                })
            }
        }
    }

    pub async fn get_account_summary(&self) -> Result<Option<VpnAccountSummary>, VpnError> {
        self.account_command_sender
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
        self.account_command_sender
            .refresh_account_state(true)
            .await
            .map_err(VpnError::from)
    }

    /// Handle a subscription payment: checks that the user is logged in, refreshes the account
    /// state.
    pub async fn handle_subscription_payment(&self) -> Result<(), VpnError> {
        self.account_command_sender
            .handle_subscription_payment()
            .await
            .map_err(VpnError::from)
    }

    /// Import the account mnemonic
    pub async fn login(&self, request: StoreAccountRequest) -> Result<(), VpnError> {
        let account =
            StorableAccount::try_from(request).map_err(|err| VpnError::InvalidMnemonic {
                details: err.to_string(),
            })?;

        self.account_command_sender.store_account(account).await?;
        Ok(())
    }

    /// Generate the account mnemonic locally and store it.
    pub async fn create_account(&self) -> Result<(), VpnError> {
        self.account_command_sender
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
            .account_command_sender
            .get_stored_account()
            .await
            .map_err(VpnError::from)?
            .ok_or(VpnError::NoAccountStored)?;
        let platform = Platform::from(request);
        self.account_command_sender
            .register_account(mnemonic, platform)
            .await
            .map_err(VpnError::from)
    }

    /// Remove the account mnemonic and all associated keys and files
    pub async fn forget_account(&self) -> Result<(), VpnError> {
        self.account_command_sender
            .forget_account()
            .await
            .map_err(VpnError::from)
    }

    /// Force a rotation of the wireguard keys
    pub async fn rotate_keys(&self) -> Result<(), VpnError> {
        self.account_command_sender
            .rotate_keys()
            .await
            .map_err(VpnError::from)
    }

    /// Get the account identity
    pub async fn get_account_identity(&self) -> Result<Option<String>, VpnError> {
        Ok(self.account_command_sender.get_account_id().await?)
    }

    /// Get the canonical account identity
    pub async fn get_canonical_account_identity(&self) -> Result<Option<String>, VpnError> {
        Ok(self
            .account_command_sender
            .get_canonical_account_id()
            .await?)
    }

    /// Get the account mode
    pub async fn get_account_mode(&self) -> Result<Option<StoredAccountMode>, VpnError> {
        Ok(self.account_command_sender.get_account_mode().await?)
    }

    /// Check if the account mnemonic is stored
    pub async fn is_account_mnemonic_stored(&self) -> Result<bool, VpnError> {
        Ok(self
            .account_command_sender
            .get_account_id()
            .await?
            .is_some())
    }

    /// Read and return the mnemonic, if there's one stored.
    pub async fn get_stored_mnemonic(&self) -> Result<String, VpnError> {
        Ok(self
            .account_command_sender
            .get_stored_account()
            .await
            .map_err(VpnError::from)?
            .ok_or(VpnError::NoAccountStored)?
            .mnemonic
            .to_string())
    }

    /// Get the device identity
    pub async fn get_device_identity(&self) -> Result<String, VpnError> {
        self.account_command_sender
            .get_device_identity()
            .await?
            .ok_or(VpnError::NoAccountStored)
    }
}
