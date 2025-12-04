// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::error::Error;
use nym_vpn_lib_types::NetworkStatisticsIdentity;
use tokio::sync::{mpsc::UnboundedSender, oneshot};
use tokio_util::sync::CancellationToken;

/// Channel receiving commands. Unlike events, they have to be handled by the Controller at all time
pub(crate) type ControllerCommandsReceiver =
    tokio::sync::mpsc::UnboundedReceiver<ControllerCommand>;

/// Channel allowing commands to be issued to the controller
#[derive(Clone)]
pub struct StatisticsCommandsSender {
    commands_tx: UnboundedSender<ControllerCommand>,
    cancel_token: CancellationToken,
}

impl StatisticsCommandsSender {
    /// Create a new statistics command Sender
    pub(crate) fn new(
        commands_tx: UnboundedSender<ControllerCommand>,
        cancel_token: CancellationToken,
    ) -> Self {
        StatisticsCommandsSender {
            commands_tx,
            cancel_token,
        }
    }

    /// Send a command
    fn send(&self, command: ControllerCommand) {
        if let Err(err) = self.commands_tx.send(command)
            && !self.cancel_token.is_cancelled()
        {
            tracing::error!("Failed to send stats event: {err}");
        }
    }

    pub fn set_enable_collection(&self, enabled: bool) {
        if enabled {
            self.send(ControllerCommand::Config(ConfigCommand::EnableCollection))
        } else {
            self.send(ControllerCommand::Config(ConfigCommand::DisableCollection))
        }
    }

    pub fn set_allow_direct_sending(&self, allow: bool) {
        self.send(ControllerCommand::Config(
            ConfigCommand::AllowDirectSending(allow),
        ))
    }

    pub async fn reset_seed(&self, seed: Option<String>) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.send(ControllerCommand::Seed(SeedCommand::ResetSeed(tx, seed)));
        rx.await.map_err(Error::internal)?
    }

    pub async fn get_seed(&self) -> Result<NetworkStatisticsIdentity, Error> {
        let (tx, rx) = oneshot::channel();
        self.send(ControllerCommand::Seed(SeedCommand::GetSeed(tx)));
        rx.await.map_err(Error::internal)?
    }
}

#[derive(Debug)]
pub(crate) enum ControllerCommand {
    Config(ConfigCommand),
    Seed(SeedCommand),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ConfigCommand {
    EnableCollection,
    DisableCollection,
    AllowDirectSending(bool),
}

#[derive(Debug)]
pub(crate) enum SeedCommand {
    ResetSeed(oneshot::Sender<Result<(), Error>>, Option<String>),
    GetSeed(oneshot::Sender<Result<NetworkStatisticsIdentity, Error>>),
}
