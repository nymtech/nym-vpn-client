// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use nym_vpn_proto::rpc_client::RpcClient;

use crate::boolean_option::BooleanOption;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get current status of anonymous network statistics collection
    Get,

    /// Enable or disable anonymous network statistics collection
    Set {
        /// Enable or disable anonymous network statistics collection
        #[arg(value_parser = clap::value_parser!(BooleanOption))]
        enable: BooleanOption,
    },
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Get => {
                let enabled = rpc_client.is_collect_network_stats_enabled().await?;
                println!(
                    "Anonymous network statistics collection: {}",
                    if enabled { "on" } else { "off" }
                );
                Ok(())
            }
            Command::Set { enable } => {
                if *enable {
                    rpc_client.enable_collect_network_stats().await?;
                } else {
                    rpc_client.disable_collect_network_stats().await?;
                }
                println!(
                    "Anonymous network statistics collection: {}",
                    if *enable { "on" } else { "off" }
                );
                println!("You have to restart the daemon for the changes to take effect.");
                Ok(())
            }
        }
    }
}
