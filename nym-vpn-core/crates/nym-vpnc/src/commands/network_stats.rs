// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use nym_vpn_proto::rpc_client::RpcClient;

use crate::{
    boolean_option::BooleanOption,
    display_helpers::{Rows, display_on_off},
};

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get current status of anonymous network statistics collection
    Get,

    /// Update network statistics collection config
    Set(SetParams),

    #[clap(hide = true)]
    /// Stats ID seed control
    Seed {
        #[command(subcommand)]
        subcommand: SeedCommand,
    },
}

#[derive(Debug, Clone, clap::Args)]
#[group(required = true, multiple = true)]
pub struct SetParams {
    /// Enable or disable anonymous network statistics collection
    #[arg(long, value_parser = clap::value_parser!(BooleanOption))]
    enabled: Option<BooleanOption>,

    /// Allow reporting stats even if not connected to the tunnel
    #[arg(long, value_parser = clap::value_parser!(BooleanOption))]
    allow_disconnected: Option<BooleanOption>,
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Get => {
                let config = rpc_client.get_config().await?;
                let identity = rpc_client.network_stats_get_seed().await?;
                Rows::new()
                    .row(
                        "Network statistics",
                        display_on_off(config.network_stats.enabled),
                    )
                    .row(
                        "Report when disconnected",
                        display_on_off(config.network_stats.allow_disconnected),
                    )
                    .row("Seed", identity.seed)
                    .row("ID", identity.id)
                    .print();

                Ok(())
            }
            Command::Set(SetParams {
                enabled,
                allow_disconnected,
            }) => {
                if let Some(enabled) = enabled {
                    rpc_client.network_stats_set_enabled(*enabled).await?;
                }

                if let Some(allow_disconnected) = allow_disconnected {
                    rpc_client
                        .network_stats_allow_disconnected(*allow_disconnected)
                        .await?;
                }

                Ok(())
            }
            Command::Seed { subcommand } => subcommand.execute(rpc_client).await,
        }
    }
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum SeedCommand {
    /// Get current used seed
    Get,

    /// Set a custom seed
    Set {
        /// Seed
        #[arg(index = 1)]
        seed: String,
    },

    /// Regenerate a new seed
    Reset,
}

impl SeedCommand {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            SeedCommand::Get => {
                let identity = rpc_client.network_stats_get_seed().await?;
                Rows::new()
                    .row("Seed", identity.seed)
                    .row("ID", identity.id)
                    .print();

                Ok(())
            }
            SeedCommand::Set { seed } => {
                Ok(rpc_client.network_stats_reset_seed(Some(seed)).await?)
            }
            SeedCommand::Reset => Ok(rpc_client.network_stats_reset_seed(None).await?),
        }
    }
}
