// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use nym_vpn_proto::rpc_client::RpcClient;

use crate::boolean_option::BooleanOption;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get current local network access policy
    Get,

    /// Set local network access policy
    Set {
        /// Allow or disallow local network access
        #[arg(value_parser = BooleanOption::custom_parser("allow", "block"))]
        allow: BooleanOption,
    },
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Get => {
                let config = rpc_client.get_config().await?;
                println!(
                    "Local network policy: {}",
                    if config.allow_lan { "allow" } else { "block" }
                );
                Ok(())
            }
            Command::Set { allow } => {
                rpc_client.set_allow_lan(*allow).await?;
                Ok(())
            }
        }
    }
}
