// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, anyhow};
use nym_vpn_proto::rpc_client::RpcClient;
use std::net::IpAddr;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get Custom DNS servers
    Get,

    /// Set Custom DNS servers (space separated)
    Set { dns_servers: Vec<String> },

    /// Clear Custom DNS servers
    Clear,

    /// Get the Default DNS servers
    GetDefault,
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
                let ip_addr_list = dns_servers
                    .iter()
                    .map(|s| {
                        s.parse()
                            .map_err(|e| anyhow!("Failed to parse '{s}' as an IP address: {e}",))
                    })
                    .collect::<Result<Vec<IpAddr>>>()?;
                rpc_client.set_custom_dns(ip_addr_list).await?;
                Ok(())
            }
            Command::Clear => {
                rpc_client.set_custom_dns(vec![]).await?;
                Ok(())
            }
            Command::GetDefault => {
                let default_dns = rpc_client.get_default_dns().await?;
                println!(
                    "Default DNS: {}",
                    default_dns
                        .iter()
                        .map(|ip| ip.to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                );
                Ok(())
            }
        }
    }
}
