// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use nym_vpn_proto::rpc_client::RpcClient;

use crate::boolean_option::BooleanOption;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get airporting settings
    Get,

    /// Set airporting settings
    Set {
        /// Enable or disable airporting
        #[arg(value_parser = BooleanOption::custom_parser("enable", "disable"))]
        enable: BooleanOption,
    },
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Get => {
                let config = rpc_client.get_config().await?;
                println!(
                    "Airporting: {}",
                    if config.enable_airporting {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
                Ok(())
            }
            Command::Set { enable } => {
                rpc_client.set_enable_airporting(*enable).await?;
                Ok(())
            }
        }
    }
}
