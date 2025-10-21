// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use nym_vpn_proto::rpc_client::RpcClient;

use crate::{boolean_option::BooleanOption, display_helpers::display_on_off};

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get current status of Sentry integration
    Get,

    /// Enable or disable Sentry integration
    Set {
        /// Enable or disable Sentry integration
        #[arg(value_parser = clap::value_parser!(BooleanOption))]
        enable: BooleanOption,
    },
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Get => {
                let enabled = rpc_client.is_sentry_enabled().await?;

                println!("Sentry integration: {}", display_on_off(enabled));
                Ok(())
            }
            Command::Set { enable } => {
                if *enable {
                    rpc_client.enable_sentry().await?;
                } else {
                    rpc_client.disable_sentry().await?;
                }
                println!("Sentry integration: {enable}");
                println!("You have to restart the daemon for the changes to take effect");
                Ok(())
            }
        }
    }
}
