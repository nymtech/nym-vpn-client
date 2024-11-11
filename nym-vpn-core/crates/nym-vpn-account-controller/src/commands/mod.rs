// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::{
    response::NymVpnDevice,
    types::{Device, VpnApiAccount},
    VpnApiClient,
};
use tokio::sync::oneshot;

use crate::{
    controller::{AccountSummaryResponse, DevicesResponse, PendingCommands},
    error::Error,
    AvailableTicketbooks, SharedAccountState,
};

pub(crate) mod register_device;
pub(crate) mod update_state;
pub(crate) mod zknym;

#[derive(Debug)]
pub enum AccountCommand {
    UpdateAccountState,
    RegisterDevice,
    RequestZkNym,
    GetDeviceZkNym,
    GetZkNymsAvailableForDownload,
    GetZkNymById(String),
    GetAvailableTickets(oneshot::Sender<Result<AvailableTicketbooks, Error>>),
}

impl AccountCommand {
    // TODO: use strum crate
    fn kind(&self) -> &'static str {
        match self {
            AccountCommand::UpdateAccountState => "update_account_state",
            AccountCommand::RegisterDevice => "register_device",
            AccountCommand::RequestZkNym => "request_zk_nym",
            AccountCommand::GetDeviceZkNym => "get_device_zk_nym",
            AccountCommand::GetZkNymsAvailableForDownload => "get_zk_nyms_available_for_download",
            AccountCommand::GetZkNymById(_) => "get_zk_nym_by_id",
            AccountCommand::GetAvailableTickets(_) => "get_available_tickets",
        }
    }
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum AccountCommandError {
    #[error("failed to get available tickets: {0}")]
    GetAvailableTickets(String),
}

#[derive(Clone, Debug)]
pub(crate) enum AccountCommandResult {
    UpdatedAccountState,
    RegisteredDevice(NymVpnDevice),
}

pub(crate) struct CommandHandler {
    id: uuid::Uuid,
    command: AccountCommand,

    account: VpnApiAccount,
    device: Device,
    pending_command: PendingCommands,
    account_state: SharedAccountState,
    vpn_api_client: VpnApiClient,

    last_account_summary: AccountSummaryResponse,
    last_devices: DevicesResponse,
}

impl CommandHandler {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        command: AccountCommand,
        account: VpnApiAccount,
        device: Device,
        pending_command: PendingCommands,
        account_state: SharedAccountState,
        vpn_api_client: VpnApiClient,
        last_account_summary: AccountSummaryResponse,
        last_devices: DevicesResponse,
    ) -> Self {
        let id = uuid::Uuid::new_v4();
        pending_command
            .lock()
            .map(|mut guard| guard.insert(id, command.kind().to_owned()))
            .map_err(|err| {
                tracing::error!(
                    "Failed to insert command {} into pending commands: {:?}",
                    id,
                    err
                )
            })
            .ok();
        tracing::debug!("Created command handler with id: {}", id);
        CommandHandler {
            id,
            command,
            account,
            device,
            pending_command,
            account_state,
            vpn_api_client,
            last_account_summary,
            last_devices,
        }
    }

    async fn update_shared_account_state(&self) -> Result<AccountCommandResult, Error> {
        let update_result = update_state::update_state(
            &self.account,
            &self.device,
            &self.account_state,
            &self.vpn_api_client,
            &self.last_account_summary,
            &self.last_devices,
        )
        .await
        .map(|_| AccountCommandResult::UpdatedAccountState);
        tracing::debug!("Current state: {:?}", self.account_state.lock().await);
        update_result
    }

    async fn register_device(&self) -> Result<AccountCommandResult, Error> {
        register_device::register_device(
            &self.account,
            &self.device,
            &self.account_state,
            &self.vpn_api_client,
        )
        .await
        .map(AccountCommandResult::RegisteredDevice)
    }

    pub(crate) async fn run(self) -> Result<AccountCommandResult, Error> {
        tracing::debug!("Running command {:?} with id {}", self.command, self.id);
        match self.command {
            AccountCommand::UpdateAccountState => self.update_shared_account_state().await,
            AccountCommand::RegisterDevice => self.register_device().await,
            AccountCommand::RequestZkNym => todo!(),
            AccountCommand::GetDeviceZkNym => todo!(),
            AccountCommand::GetZkNymsAvailableForDownload => todo!(),
            AccountCommand::GetZkNymById(_) => todo!(),
            AccountCommand::GetAvailableTickets(_) => todo!(),
        }
        .inspect(|_result| {
            tracing::info!("Command {:?} with id {} completed", self.command, self.id);
        })
        .inspect_err(|err| {
            tracing::warn!(
                "Command {:?} with id {} completed with error",
                self.command,
                self.id
            );
            tracing::debug!(
                "Command {:?} with id {} failed with error: {:?}",
                self.command,
                self.id,
                err
            );
        })
    }
}

impl Drop for CommandHandler {
    fn drop(&mut self) {
        self.pending_command
            .lock()
            .map(|mut guard| guard.remove(&self.id))
            .inspect_err(|err| {
                tracing::error!(
                    "Failed to remove command {} from pending commands: {:?}",
                    self.id,
                    err
                )
            })
            .ok();
    }
}
