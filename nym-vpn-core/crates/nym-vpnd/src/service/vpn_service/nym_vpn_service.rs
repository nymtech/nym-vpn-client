// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use bip39::Mnemonic;
use futures::{channel::mpsc::UnboundedSender, SinkExt};
use nym_vpn_lib::{
    credentials::import_credential,
    gateway_directory::{self, EntryPoint, ExitPoint},
    GenericNymVpnConfig, MixnetClientConfig,
};
use nym_vpn_store::keys::KeyStore as _;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::sync::{broadcast, mpsc::UnboundedReceiver};
use tracing::{debug, error, info};

use crate::service::{
    config::{ConfigSetupError, NymVpnServiceConfig},
    exit_listener::VpnServiceExitListener,
    status_listener::VpnServiceStatusListener,
    vpn_service::response::VpnServiceConnectHandle,
    ImportCredentialError, StoreAccountError,
};

use super::{
    ConnectArgs, SharedVpnState, VpnServiceCommand, VpnServiceConnectResult,
    VpnServiceDisconnectResult, VpnServiceInfoResult, VpnServiceStateChange,
    VpnServiceStatusResult, VpnState,
};

pub(crate) struct NymVpnService<S>
where
    S: nym_vpn_store::VpnStorage,
{
    shared_vpn_state: SharedVpnState,

    // Listen for commands from the command interface, like the grpc listener that listens user
    // commands.
    vpn_command_rx: UnboundedReceiver<VpnServiceCommand>,

    // Send commands to the actual vpn service task
    vpn_ctrl_sender: Option<UnboundedSender<nym_vpn_lib::NymVpnCtrlMessage>>,

    config_file: PathBuf,

    data_dir: PathBuf,

    // Storage backend
    storage: S,
}

impl NymVpnService<nym_vpn_lib::storage::VpnClientOnDiskStorage> {
    pub(crate) fn new(
        vpn_state_changes_tx: broadcast::Sender<VpnServiceStateChange>,
        vpn_command_rx: UnboundedReceiver<VpnServiceCommand>,
    ) -> Self {
        let config_dir = std::env::var("NYM_VPND_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| crate::service::config::default_config_dir());
        let config_file = config_dir.join(crate::service::config::DEFAULT_CONFIG_FILE);
        let data_dir = std::env::var("NYM_VPND_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| crate::service::config::default_data_dir());
        let storage = nym_vpn_lib::storage::VpnClientOnDiskStorage::new(data_dir.clone());
        Self {
            shared_vpn_state: SharedVpnState::new(vpn_state_changes_tx),
            vpn_command_rx,
            vpn_ctrl_sender: None,
            config_file,
            data_dir,
            storage,
        }
    }

    pub(crate) async fn init_storage(&self) -> Result<(), ConfigSetupError> {
        // Make sure the data dir exists
        if let Err(err) = crate::service::config::create_data_dir(&self.data_dir) {
            self.shared_vpn_state.set(VpnState::NotConnected);
            return Err(err);
        }

        // Generate the device keys if we don't already have them
        if let Err(err) = self.storage.init_keys(None).await {
            self.shared_vpn_state.set(VpnState::NotConnected);
            return Err(ConfigSetupError::FailedToInitKeys { source: err });
        }

        Ok(())
    }
}

impl<S> NymVpnService<S>
where
    S: nym_vpn_store::VpnStorage,
{
    fn try_setup_config(
        &self,
        entry: Option<gateway_directory::EntryPoint>,
        exit: Option<gateway_directory::ExitPoint>,
    ) -> std::result::Result<NymVpnServiceConfig, ConfigSetupError> {
        // If the config file does not exit, create it
        let config = if self.config_file.exists() {
            let mut read_config = crate::service::config::read_config_file(&self.config_file)
                .map_err(|err| {
                    error!(
                        "Failed to read config file, resetting to defaults: {:?}",
                        err
                    );
                })
                .unwrap_or_default();
            read_config.entry_point = entry.unwrap_or(read_config.entry_point);
            read_config.exit_point = exit.unwrap_or(read_config.exit_point);
            crate::service::config::write_config_file(&self.config_file, &read_config)?;
            read_config
        } else {
            let config = NymVpnServiceConfig {
                entry_point: entry.unwrap_or(EntryPoint::Random),
                exit_point: exit.unwrap_or(ExitPoint::Random),
            };
            crate::service::config::create_config_file(&self.config_file, config)?
        };
        Ok(config)
    }

    async fn handle_connect(&mut self, connect_args: ConnectArgs) -> VpnServiceConnectResult {
        self.shared_vpn_state.set(VpnState::Connecting);

        let ConnectArgs {
            entry,
            exit,
            options,
        } = connect_args;

        info!(
            "Using entry point: {}",
            entry
                .clone()
                .map(|e| e.to_string())
                .unwrap_or("None".to_string())
        );
        info!(
            "Using exit point: {}",
            exit.clone()
                .map(|e| e.to_string())
                .unwrap_or("None".to_string())
        );
        info!("Using options: {:?}", options);

        let config = match self.try_setup_config(entry, exit) {
            Ok(config) => config,
            Err(err) => {
                self.shared_vpn_state.set(VpnState::NotConnected);
                return VpnServiceConnectResult::Fail(err.to_string());
            }
        };

        info!("Using config: {}", config);

        let generic_config = GenericNymVpnConfig {
            mixnet_client_config: MixnetClientConfig {
                enable_poisson_rate: options.enable_poisson_rate,
                disable_background_cover_traffic: options.disable_background_cover_traffic,
                enable_credentials_mode: options.enable_credentials_mode,
                min_mixnode_performance: options.min_mixnode_performance,
                min_gateway_performance: options.min_gateway_performance,
            },
            data_path: Some(self.data_dir.clone()),
            gateway_config: gateway_directory::Config::new_from_env(
                options.min_gateway_performance,
            ),
            entry_point: config.entry_point.clone(),
            exit_point: config.exit_point.clone(),
            nym_ips: None,
            nym_mtu: None,
            dns: options.dns,
            disable_routing: options.disable_routing,
            user_agent: Some(nym_bin_common::bin_info_local_vergen!().into()),
        };

        let nym_vpn = if options.enable_two_hop {
            let mut nym_vpn =
                nym_vpn_lib::NymVpn::new_wireguard_vpn(config.entry_point, config.exit_point);
            nym_vpn.generic_config = generic_config;
            nym_vpn.into()
        } else {
            let mut nym_vpn =
                nym_vpn_lib::NymVpn::new_mixnet_vpn(config.entry_point, config.exit_point);
            nym_vpn.generic_config = generic_config;
            nym_vpn.into()
        };

        let handle = nym_vpn_lib::spawn_nym_vpn_with_new_runtime(nym_vpn).unwrap();

        let nym_vpn_lib::NymVpnHandle {
            vpn_ctrl_tx,
            vpn_status_rx,
            vpn_exit_rx,
        } = handle;

        self.vpn_ctrl_sender = Some(vpn_ctrl_tx);

        let (listener_vpn_status_tx, listener_vpn_status_rx) = futures::channel::mpsc::channel(16);
        let (listener_vpn_exit_tx, listener_vpn_exit_rx) = futures::channel::oneshot::channel();

        VpnServiceStatusListener::new(self.shared_vpn_state.clone())
            .start(vpn_status_rx, listener_vpn_status_tx)
            .await;

        VpnServiceExitListener::new(self.shared_vpn_state.clone())
            .start(vpn_exit_rx, listener_vpn_exit_tx)
            .await;

        let connect_handle = VpnServiceConnectHandle {
            listener_vpn_status_rx,
            listener_vpn_exit_rx,
        };

        VpnServiceConnectResult::Success(connect_handle)
    }

    fn is_running(&self) -> bool {
        self.vpn_ctrl_sender
            .as_ref()
            .map(|s| !s.is_closed())
            .unwrap_or(false)
    }

    async fn handle_disconnect(&mut self) -> VpnServiceDisconnectResult {
        // To handle the mutable borrow we set the state separate from the sending the stop message
        if self.is_running() {
            self.shared_vpn_state.set(VpnState::Disconnecting);
        } else {
            return VpnServiceDisconnectResult::NotRunning;
        }

        if let Some(ref mut vpn_ctrl_sender) = self.vpn_ctrl_sender {
            vpn_ctrl_sender
                .send(nym_vpn_lib::NymVpnCtrlMessage::Stop)
                .await
                .ok();
            VpnServiceDisconnectResult::Success
        } else {
            VpnServiceDisconnectResult::NotRunning
        }
    }

    async fn handle_status(&self) -> VpnServiceStatusResult {
        self.shared_vpn_state.get().into()
    }

    async fn handle_info(&self) -> VpnServiceInfoResult {
        let network = nym_vpn_lib::nym_config::defaults::NymNetworkDetails::new_from_env();
        let bin_info = nym_bin_common::bin_info_local_vergen!();
        VpnServiceInfoResult {
            version: bin_info.build_version.to_string(),
            build_timestamp: time::OffsetDateTime::parse(bin_info.build_timestamp, &Rfc3339).ok(),
            triple: bin_info.cargo_triple.to_string(),
            git_commit: bin_info.commit_sha.to_string(),
            network_name: network.network_name,
            endpoints: network.endpoints,
            nym_vpn_api_url: network.nym_vpn_api_url,
        }
    }

    async fn handle_import_credential(
        &mut self,
        credential: Vec<u8>,
    ) -> Result<Option<OffsetDateTime>, ImportCredentialError> {
        if self.is_running() {
            return Err(ImportCredentialError::VpnRunning);
        }

        import_credential(credential, self.data_dir.clone())
            .await
            .map_err(|err| err.into())
    }

    async fn handle_store_account(&mut self, account: String) -> Result<(), StoreAccountError>
    where
        <S as nym_vpn_store::mnemonic::MnemonicStorage>::StorageError: Sync + Send + 'static,
    {
        self.storage
            .store_mnemonic(Mnemonic::parse(&account)?)
            .await
            .map_err(|err| StoreAccountError::FailedToStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn run(mut self) -> anyhow::Result<()>
    where
        <S as nym_vpn_store::mnemonic::MnemonicStorage>::StorageError: Sync + Send + 'static,
    {
        while let Some(command) = self.vpn_command_rx.recv().await {
            debug!("VPN: Received command: {command}");
            match command {
                VpnServiceCommand::Connect(tx, connect_args) => {
                    let result = self.handle_connect(connect_args).await;
                    tx.send(result).unwrap();
                }
                VpnServiceCommand::Disconnect(tx) => {
                    let result = self.handle_disconnect().await;
                    tx.send(result).unwrap();
                }
                VpnServiceCommand::Status(tx) => {
                    let result = self.handle_status().await;
                    tx.send(result).unwrap();
                }
                VpnServiceCommand::Info(tx) => {
                    let result = self.handle_info().await;
                    tx.send(result).unwrap();
                }
                VpnServiceCommand::ImportCredential(tx, credential) => {
                    let result = self.handle_import_credential(credential).await;
                    tx.send(result).unwrap();
                }
                VpnServiceCommand::StoreAccount(tx, account) => {
                    let result = self.handle_store_account(account).await;
                    tx.send(result).unwrap();
                }
            }
        }
        Ok(())
    }
}
