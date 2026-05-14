// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use clap::builder::ValueParserFactory;

use nym_vpn_proto::rpc_client::RpcClient;

use crate::boolean_option::BooleanOption;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get current Ad blocking state
    Get,

    /// Set Ad blocking state
    Set {
        /// Enable or disable Ad blocking
        #[arg(value_parser = BooleanOption::value_parser())]
        enabled: BooleanOption,
    },
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Get => {
                let config = rpc_client.get_config().await?;
                println!(
                    "Ad blocking: {}",
                    if config.enable_ad_blocking {
                        "on"
                    } else {
                        "off"
                    }
                );
                Ok(())
            }
            Command::Set { enabled } => {
                rpc_client.set_enable_ad_blocking(*enabled).await?;
                Ok(())
            }
        }
    }
}
