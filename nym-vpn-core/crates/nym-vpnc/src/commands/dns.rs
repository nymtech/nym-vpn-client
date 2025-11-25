// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use nym_vpn_proto::rpc_client::RpcClient;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get Custom DNS servers
    Get,

    /// Set Custom DNS servers (space separated)
    Set { dns_servers: Vec<String> },

    /// Clear Custom DNS servers
    Clear,
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Get => {
                let config = rpc_client.get_config().await?;
                println!(
                    "Custom DNS: {}",
                    config
                        .custom_dns
                        .map(|dns| dns
                            .iter()
                            .map(|ip| ip.to_string())
                            .collect::<Vec<_>>()
                            .join(" "))
                        .unwrap_or_else(|| "not set".to_string())
                );
                Ok(())
            }
            Command::Set { dns_servers } => {
                rpc_client.set_custom_dns(Some(dns_servers)).await?;
                Ok(())
            }
            Command::Clear => {
                rpc_client.set_custom_dns(None).await?;
                Ok(())
            }
        }
    }
}
