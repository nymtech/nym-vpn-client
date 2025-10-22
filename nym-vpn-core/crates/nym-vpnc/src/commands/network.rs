// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use nym_vpn_proto::rpc_client::RpcClient;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get current Nym network.
    Get,

    /// Set Nym network.
    Set {
        /// Network name (i.e: mainnet, sandbox, canary, evil)
        #[arg(index = 1)]
        network_name: String,
    },
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Get => {
                let network_name = rpc_client.get_info().await?.nym_network.network_name;
                println!("Current network: {network_name}");
                Ok(())
            }
            Command::Set { network_name } => {
                rpc_client.set_network(network_name).await?;
                println!(
                    "Network changed. You have to restart the daemon for the changes to take effect."
                );
                Ok(())
            }
        }
    }
}
