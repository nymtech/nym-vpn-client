// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, sync::Arc};

use super::AccountCommand;

#[derive(Debug, Default)]
pub(crate) struct RunningCommands {
    running_commands: Arc<tokio::sync::Mutex<HashMap<String, Vec<AccountCommand>>>>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Command {
    IsFirst,
    IsNotFirst,
}

// Add the command to the set of running commands.
// Returns true if this is the first command of this type, otherwise false.
impl RunningCommands {
    pub(crate) async fn add(&self, command: AccountCommand) -> Command {
        let mut running_commands = self.running_commands.lock().await;
        let commands = running_commands.entry(command.kind()).or_default();
        let is_first = if commands.is_empty() {
            Command::IsFirst
        } else {
            Command::IsNotFirst
        };
        commands.push(command);
        is_first
    }

    pub(crate) async fn remove(&self, command: &AccountCommand) -> Vec<AccountCommand> {
        let mut running_commands = self.running_commands.lock().await;
        let removed_commands = running_commands.remove(&command.kind());
        removed_commands.unwrap_or_default()
    }
}
