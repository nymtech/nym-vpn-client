// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use nym_vpn_proto::rpc_client::RpcClient;

use crate::boolean_option::BooleanOption;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Display current tunnel configuration
    Get,

    /// Update tunnel configuration
    Set {
        #[arg(value_parser = BooleanOption::custom_parser("on", "off"))]
        enable_ipv6: Option<BooleanOption>,

        #[arg(value_parser = BooleanOption::custom_parser("on", "off"))]
        enable_two_hop: Option<BooleanOption>,

        #[arg(value_parser = BooleanOption::custom_parser("on", "off"))]
        netstack: Option<BooleanOption>,
    },
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Get => {
                let config = rpc_client.get_config().await?;
                println!("IPv6: {}", if config.disable_ipv6 { "off" } else { "on" });
                println!(
                    "Two-hop: {}",
                    if config.enable_two_hop { "on" } else { "off" }
                );
                println!("Netstack: {}", if config.netstack { "on" } else { "off" });

                Ok(())
            }
            Command::Set {
                enable_two_hop,
                netstack,
                enable_ipv6,
            } => {
                if let Some(enable_two_hop) = enable_two_hop {
                    rpc_client.set_enable_two_hop(*enable_two_hop).await?;
                }

                if let Some(netstack) = netstack {
                    rpc_client.set_netstack(*netstack).await?;
                }

                if let Some(enable_ipv6) = enable_ipv6 {
                    rpc_client.set_disable_ipv6(!*enable_ipv6).await?;
                }

                Ok(())
            }
        }
    }
}
