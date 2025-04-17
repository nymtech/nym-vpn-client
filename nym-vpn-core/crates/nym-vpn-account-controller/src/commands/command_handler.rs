// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_vpn_api_client::{
    response::{NymVpnAccountSummaryResponse, NymVpnDevice},
    types::{Device, VpnApiAccount},
};
use nym_vpn_lib_types::{
    RegisterDeviceError, RequestZkNymError, SyncAccountError, SyncDeviceError,
};
use tokio::task::{JoinError, JoinSet};

use crate::{
    commands::{
        tasks::{
            register_device::RegisterDeviceCommandHandler,
            request_zknym::{RequestZkNymSummary, WaitingRequestZkNymCommandHandler},
            sync_account::WaitingSyncAccountCommandHandler,
            sync_device::WaitingSyncDeviceCommandHandler,
        },
        AccountCommand, AccountCommandResult, Command, RunningCommands,
    },
    connectivity::OfflineWatch,
    shared_state::DeviceState,
    storage::VpnCredentialStorage,
    SharedAccountState,
};

pub(crate) struct AccountCommandHandler {
    // List of currently running commands and their type
    running_commands: RunningCommands,

    // The task handles of the currently running commands
    running_command_tasks: JoinSet<AccountCommandResult>,

    // Account sync command handler state reused between runs
    waiting_sync_account_command_handler: WaitingSyncAccountCommandHandler,

    // Device sync command handler state reused between runs
    waiting_sync_device_command_handler: WaitingSyncDeviceCommandHandler,

    // Zk-nym request command handler state reused between runs
    waiting_request_zknym_command_handler: WaitingRequestZkNymCommandHandler,
}

impl AccountCommandHandler {
    pub(crate) fn new(
        account_state: SharedAccountState,
        vpn_api_client: nym_vpn_api_client::VpnApiClient,
        credential_storage: Arc<tokio::sync::Mutex<VpnCredentialStorage>>,
        offline_watch: OfflineWatch,
    ) -> Self {
        let waiting_sync_account_command_handler =
            WaitingSyncAccountCommandHandler::new(account_state.clone(), vpn_api_client.clone());
        let waiting_sync_device_command_handler =
            WaitingSyncDeviceCommandHandler::new(account_state.clone(), vpn_api_client.clone());
        let waiting_request_zknym_command_handler = WaitingRequestZkNymCommandHandler::new(
            credential_storage,
            account_state.clone(),
            vpn_api_client.clone(),
            offline_watch,
        );

        Self {
            running_commands: Default::default(),
            running_command_tasks: JoinSet::new(),
            waiting_sync_account_command_handler,
            waiting_sync_device_command_handler,
            waiting_request_zknym_command_handler,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.waiting_request_zknym_command_handler.reset();
    }

    pub(crate) fn is_idle(&self) -> bool {
        self.running_command_tasks.is_empty()
    }

    pub(crate) fn is_command_running(&self) -> bool {
        !self.is_idle()
    }

    pub(crate) async fn join_next(&mut self) -> Option<Result<AccountCommandResult, JoinError>> {
        self.running_command_tasks.join_next().await
    }

    pub(crate) fn try_join_next(&mut self) -> Option<Result<AccountCommandResult, JoinError>> {
        self.running_command_tasks.try_join_next()
    }

    // The idle commands handlers have vpn api clients that can be updated, should the
    // configuration change.
    pub(crate) fn update_vpn_api_client(
        &mut self,
        vpn_api_client: nym_vpn_api_client::VpnApiClient,
    ) {
        self.waiting_sync_account_command_handler
            .update_vpn_api_client(vpn_api_client.clone());
        self.waiting_sync_device_command_handler
            .update_vpn_api_client(vpn_api_client.clone());
        self.waiting_request_zknym_command_handler
            .update_vpn_api_client(vpn_api_client);
    }

    async fn spawn(
        &mut self,
        command: AccountCommand,
        task: impl std::future::Future<Output = AccountCommandResult> + Send + 'static,
    ) {
        if self.running_commands.add(command).await == Command::IsFirst {
            self.running_command_tasks.spawn(task);
        }
    }
}

// ----------------------------------------------------------------------------
// The commands that the handler can be instructed to run.
// ----------------------------------------------------------------------------

impl AccountCommandHandler {
    // ----------------------------------------------------------------------------
    // Sync the account state
    // ----------------------------------------------------------------------------

    pub(crate) async fn sync_account_state(
        &mut self,
        command: AccountCommand,
        account: VpnApiAccount,
    ) {
        if !matches!(command, AccountCommand::SyncAccountState(_)) {
            tracing::error!("Invalid command type for sync account state: {command}");
            return;
        }

        let command_handler = self.waiting_sync_account_command_handler.build(account);

        self.spawn(command, command_handler.run()).await;
    }

    pub(crate) async fn finish_sync_account_state(
        &self,
        result: &Result<NymVpnAccountSummaryResponse, SyncAccountError>,
    ) {
        let commands = self
            .running_commands
            .remove(&AccountCommand::SyncAccountState(None))
            .await;

        for command in commands {
            if let AccountCommand::SyncAccountState(Some(tx)) = command {
                tx.send(result.clone());
            }
        }
    }

    // ----------------------------------------------------------------------------
    // Sync the device state
    // ----------------------------------------------------------------------------

    pub(crate) async fn sync_device_state(
        &mut self,
        command: AccountCommand,
        account: VpnApiAccount,
        device: Device,
    ) {
        if !matches!(command, AccountCommand::SyncDeviceState(_)) {
            tracing::error!("Invalid command type for sync device state: {command}");
            return;
        }

        let command_handler = self
            .waiting_sync_device_command_handler
            .build(account, device);

        self.spawn(command, command_handler.run()).await;
    }

    pub(crate) async fn finish_sync_device_state(
        &self,
        result: &Result<DeviceState, SyncDeviceError>,
    ) {
        let commands = self
            .running_commands
            .remove(&AccountCommand::SyncDeviceState(None))
            .await;

        for command in commands {
            if let AccountCommand::SyncDeviceState(Some(tx)) = command {
                tx.send(result.clone());
            }
        }
    }

    // ----------------------------------------------------------------------------
    // Register a new device
    // ----------------------------------------------------------------------------

    pub(crate) async fn register_device(
        &mut self,
        command: AccountCommand,
        account: VpnApiAccount,
        device: Device,
        account_state: SharedAccountState,
        vpn_api_client: nym_vpn_api_client::VpnApiClient,
    ) {
        if !matches!(command, AccountCommand::RegisterDevice(_)) {
            tracing::error!("Invalid command type for register device: {command}");
            return;
        }

        let command_handler =
            RegisterDeviceCommandHandler::new(account, device, account_state, vpn_api_client);

        self.spawn(command, command_handler.run()).await;
    }

    pub(crate) async fn finish_register_device(
        &self,
        result: &Result<NymVpnDevice, RegisterDeviceError>,
    ) {
        let commands = self
            .running_commands
            .remove(&AccountCommand::RegisterDevice(None))
            .await;

        for command in commands {
            if let AccountCommand::RegisterDevice(Some(tx)) = command {
                tx.send(result.clone());
            }
        }
    }

    // ----------------------------------------------------------------------------
    // Request zk-nym ticketbooks
    // ----------------------------------------------------------------------------

    pub(crate) async fn request_zk_nym(
        &mut self,
        command: AccountCommand,
        account: VpnApiAccount,
        device: Device,
    ) {
        if !matches!(command, AccountCommand::RequestZkNym(_)) {
            tracing::error!("Invalid command type for request zk-nym: {command}");
            return;
        }

        let command_handler = self
            .waiting_request_zknym_command_handler
            .build(account, device);

        self.spawn(command, command_handler.run()).await;
    }

    pub(crate) async fn finish_request_zk_nym(
        &self,
        result: Result<RequestZkNymSummary, RequestZkNymError>,
    ) {
        let commands = self
            .running_commands
            .remove(&AccountCommand::RequestZkNym(None))
            .await;

        for command in commands {
            if let AccountCommand::RequestZkNym(Some(tx)) = command {
                tx.send(result.clone());
            }
        }
    }

    pub(crate) async fn max_zknym_request_fails_reached(&self) -> bool {
        self.waiting_request_zknym_command_handler
            .max_fails_reached()
            .await
    }
}
