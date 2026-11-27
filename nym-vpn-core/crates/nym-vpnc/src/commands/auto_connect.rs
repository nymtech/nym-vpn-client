// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use clap::builder::ValueParserFactory;

use nym_vpn_proto::rpc_client::RpcClient;

use crate::{boolean_option::BooleanOption, display_helpers::display_on_off};

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get current auto-connect state
    Get,

    /// Set auto-connect state
    Set {
        /// Enable or disable connecting automatically when the daemon starts
        #[arg(value_parser = BooleanOption::value_parser())]
        enabled: BooleanOption,
    },
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Get => {
                let config = rpc_client.get_config().await?;
                println!("Auto-connect: {}", display_on_off(config.auto_connect));
                Ok(())
            }
            Command::Set { enabled } => {
                rpc_client.set_auto_connect(*enabled).await?;
                Ok(())
            }
        }
    }
}
