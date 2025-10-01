// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{sync::OnceLock, time::Duration};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use nym_offline_monitor::{Connectivity, ConnectivityMonitor};
use nym_vpn_account_controller::{
    AccountCommandSender, AccountController, AccountControllerConfig, AccountStateReceiver,
};
use nym_vpn_api_client::{VpnApiClient, types::VpnAccount};
use nym_vpn_lib_types::AccountControllerState;
use nym_vpn_network_config::Network;
use nym_vpn_store::{
    keys::device::{DeviceKeyStore, DeviceKeys},
    mnemonic::{AccountStorage, Mnemonic, ephemeral::InMemoryMnemonicStorageError},
};
use wiremock::{Mock, MockServer};

use tokio::{sync::watch, task::JoinHandle};
use tokio_util::sync::{CancellationToken, DropGuard};

use crate::common::credential_proxy::MockCredentialProxy;

pub mod account_summary;
pub mod credential_proxy;
pub mod endpoints;

// Ensure tracing is only initialised once for the whole test suite.
static TEST_TRACING: OnceLock<()> = OnceLock::new();

pub fn init_tracing() {
    TEST_TRACING.get_or_init(|| {
        let env_filter = if std::env::var("RUST_LOG").is_ok() {
            EnvFilter::from_default_env()
        } else {
            EnvFilter::new("info,nym_vpn_account_controller=info,nym_vpn_api_client=info")
        };

        let fmt_layer = fmt::layer()
            .with_target(false)
            .compact()
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_writer(std::io::stdout);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    });
}

pub fn mock_user_agent() -> nym_http_api_client::UserAgent {
    nym_http_api_client::UserAgent {
        application: "Always".to_string(),
        version: "Coming from".to_string(),
        platform: "Take".to_string(),
        git_commit: "Me Down".to_string(),
    }
}

pub fn mock_mnemonic() -> Mnemonic {
    Mnemonic::parse::<&str>("dash hungry rate famous lesson march suit refuse excite soul faith bid buddy tortoise melody advice dirt coffee fluid sure air decrease cargo work").unwrap()
}

pub fn mock_account_id() -> String {
    VpnAccount::try_from(mock_mnemonic())
        .unwrap()
        .id()
        .to_string()
}

/// Mock connectivity monitor to simulate offline mode
#[derive(Clone)]
pub struct MockConnectivityHandle {
    current_state: Connectivity,
    pub connectivity_channel: (watch::Sender<Connectivity>, watch::Receiver<Connectivity>),
}

impl MockConnectivityHandle {
    pub fn new() -> Self {
        let connectivity_channel = watch::channel(Connectivity::Status {
            ipv4: true,
            ipv6: true,
        });
        let current_state = *connectivity_channel.1.borrow();
        Self {
            current_state,
            connectivity_channel,
        }
    }
}

#[async_trait::async_trait]
impl ConnectivityMonitor for MockConnectivityHandle {
    async fn connectivity(&'async_trait self) -> Connectivity {
        self.current_state
    }

    async fn next(&'async_trait mut self) -> Option<Connectivity> {
        self.connectivity_channel.1.changed().await.ok()?;
        Some(*self.connectivity_channel.1.borrow_and_update())
    }
}

/// Mock storage for device keys and mnemonic
#[derive(Default)]
pub struct MockEphemeralStorage {
    key_store: nym_vpn_store::keys::device::InMemEphemeralKeys,
    mnemonic_storage: nym_vpn_store::mnemonic::ephemeral::InMemoryMnemonicStorage,
}

impl nym_vpn_store::VpnStorage for MockEphemeralStorage {}

#[async_trait::async_trait]
impl DeviceKeyStore for MockEphemeralStorage {
    type StorageError = std::convert::Infallible;

    async fn load_keys(&self) -> Result<Option<DeviceKeys>, Self::StorageError> {
        self.key_store.load_keys().await
    }

    async fn store_keys(&self, keys: &DeviceKeys) -> Result<(), Self::StorageError> {
        self.key_store.store_keys(keys).await
    }

    async fn init_keys(&self, seed: Option<[u8; 32]>) -> Result<(), Self::StorageError> {
        self.key_store.init_keys(seed).await
    }

    async fn reset_keys(&self, seed: Option<[u8; 32]>) -> Result<(), Self::StorageError> {
        self.key_store.reset_keys(seed).await
    }

    async fn remove_keys(&self) -> Result<(), Self::StorageError> {
        self.key_store.remove_keys().await
    }
}

#[async_trait::async_trait]
impl AccountStorage for MockEphemeralStorage {
    type StorageError = InMemoryMnemonicStorageError;

    async fn load_mnemonic(&self) -> Result<Option<Mnemonic>, Self::StorageError> {
        self.mnemonic_storage.load_mnemonic().await
    }

    async fn store_mnemonic(&self, mnemonic: Mnemonic) -> Result<(), Self::StorageError> {
        self.mnemonic_storage.store_mnemonic(mnemonic).await
    }

    async fn remove_mnemonic(&self) -> Result<(), Self::StorageError> {
        self.mnemonic_storage.remove_mnemonic().await
    }
}

/// Test Bench with everything needed to run an account controller
pub struct TestBench {
    /// AC handle so we don't detach the task
    _account_controller_handle: JoinHandle<()>,

    /// Temporary storage directory reference, otherwise it gets deleted
    _tempdir: tempfile::TempDir,

    /// Channel to send command to the AC
    pub command_sender: AccountCommandSender,

    /// Channel to keep track of its state
    pub state_receiver: AccountStateReceiver,

    /// Connectivity monitor to mock offline/online switch
    pub connectivity: MockConnectivityHandle,

    /// Mock VPN API server
    pub vpn_api_server: MockServer,

    /// Mock credential proxy to issue valid zk-nyms
    pub credential_proxy: MockCredentialProxy,

    /// DropGuard to stop the account controller when the testbench is dropped
    _drop_guard: DropGuard,
}

impl TestBench {
    /// Sets up a new testbench. The VPN API has no route
    pub async fn new() -> anyhow::Result<TestBench> {
        Self::new_with_credential(true).await
    }

    /// Sets up a new testbench without credential, for easier setup. The VPN API has no route
    pub async fn new_no_credentials() -> anyhow::Result<TestBench> {
        Self::new_with_credential(false).await
    }

    async fn new_with_credential(credential_enabled: bool) -> anyhow::Result<TestBench> {
        // Enable logging as early as possible so setup emits logs if needed
        init_tracing();

        // Setup storage
        let storage = MockEphemeralStorage::default();

        // Setup offline monitor
        let connectivity = MockConnectivityHandle::new();

        // Setup mock server. Route behavior has to be done by the actual tests
        let vpn_api_server = MockServer::start().await;
        let nym_vpn_api_client =
            VpnApiClient::new(vpn_api_server.uri().parse()?, mock_user_agent())?;

        let tempdir = tempfile::tempdir()?;
        let account_controller_config = AccountControllerConfig {
            data_dir: tempdir.path().to_owned(),
            credentials_mode: Some(credential_enabled),
            network_env: Network::mainnet_default().unwrap(),
        };

        let credential_proxy = MockCredentialProxy::new()?;

        let shutdown_token = CancellationToken::new();

        let account_controller = AccountController::new(
            nym_vpn_api_client,
            account_controller_config,
            storage,
            connectivity.clone(),
            shutdown_token.child_token(),
        )
        .await?;

        let command_sender = account_controller.get_command_sender();
        let state_receiver = account_controller.get_state_receiver();

        let _account_controller_handle = tokio::task::spawn(account_controller.run());

        Ok(TestBench {
            _account_controller_handle,
            _tempdir: tempdir,
            command_sender,
            state_receiver,
            connectivity,
            vpn_api_server,
            credential_proxy,
            _drop_guard: shutdown_token.drop_guard(),
        })
    }

    /// Assert that we are in a given state within 15 seconds.
    /// This is needed to avoid tokio::sleep and yield_now everywhere
    /// If after the 15sec delay, the expected state is not reached, the `assert_eq` call will fail, with a normal failure
    pub async fn assert_state(&mut self, expected_state: AccountControllerState) {
        // Make sure we're not running right away
        tokio::task::yield_now().await;

        let mut state_watcher = self.state_receiver.subscribe();

        let wait_for_state_fut = state_watcher.wait_for(|state| *state == expected_state);

        let _ = tokio::time::timeout(Duration::from_secs(15), wait_for_state_fut).await;

        // For the nice output in tests
        assert_eq!(self.state_receiver.get_state(), expected_state);
    }

    /// Tell the mock connectivity monitor to go offline
    pub fn go_offline(&self) -> anyhow::Result<()> {
        self.connectivity
            .connectivity_channel
            .0
            .send(Connectivity::Status {
                ipv4: false,
                ipv6: false,
            })?;
        Ok(())
    }

    /// Tell the mock connectivity monitor to go online
    pub fn go_online(&self) -> anyhow::Result<()> {
        self.connectivity
            .connectivity_channel
            .0
            .send(Connectivity::Status {
                ipv4: true,
                ipv6: true,
            })?;
        Ok(())
    }

    pub async fn store_mock_account(&self) -> anyhow::Result<()> {
        self.command_sender.store_account(mock_mnemonic()).await?;
        Ok(())
    }

    pub async fn forget_account(&self) -> anyhow::Result<()> {
        self.command_sender.forget_account().await?;
        Ok(())
    }

    /// Register a list of mocks with the VPN API mock server
    pub async fn register_mocks(&self, mocks: Vec<Mock>) {
        for mock in mocks {
            self.vpn_api_server.register(mock).await
        }
    }
}
