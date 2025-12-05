// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use nym_vpn_proto::rpc_client::RpcClient;

use crate::{boolean_option::BooleanOption, display_helpers::display_on_off};

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Display current tunnel configuration
    Get,

    /// Update tunnel configuration
    Set(SetParams),
}

#[derive(Debug, Clone, clap::Args)]
#[group(required = true, multiple = true)]
pub struct SetParams {
    /// Enable or disable IPv6
    #[arg(long, value_parser = clap::value_parser!(BooleanOption))]
    ipv6: Option<BooleanOption>,

    /// Enable or disable two-hop mode
    #[arg(long, value_parser = clap::value_parser!(BooleanOption))]
    two_hop: Option<BooleanOption>,

    /// Enable or disable netstack in two-hop mode
    /// Normally this is only used for testing purposes and should always be off
    #[arg(long, value_parser = clap::value_parser!(BooleanOption))]
    netstack: Option<BooleanOption>,

    /// Enable Circumvention Transport (CT) wrapping for the connection to the entry gateway in two hop wireguard mode.
    #[arg(long, alias = "ct", value_parser = clap::value_parser!(BooleanOption))]
    circumvention_transports: Option<BooleanOption>,
}

impl Command {
    pub async fn execute(self, rpc_client: &RpcClient) -> Result<()> {
        match self {
            Command::Get => {
                let config = rpc_client.get_config().await?;
                println!("IPv6: {}", display_on_off(!config.disable_ipv6));
                println!("Two-hop: {}", display_on_off(config.enable_two_hop));
                println!("Netstack: {}", display_on_off(config.netstack));
                println!(
                    "Circumvention transports: {}",
                    display_on_off(config.enable_bridges)
                );

                Ok(())
            }
            Command::Set(SetParams {
                two_hop,
                netstack,
                ipv6,
                circumvention_transports,
            }) => {
                if let Some(two_hop) = two_hop {
                    rpc_client.set_enable_two_hop(*two_hop).await?;
                }

                if let Some(netstack) = netstack {
                    rpc_client.set_netstack(*netstack).await?;
                }

                if let Some(ipv6) = ipv6 {
                    rpc_client.set_disable_ipv6(!*ipv6).await?;
                }

                if let Some(enable_ct) = circumvention_transports {
                    rpc_client.set_enable_bridges(*enable_ct).await?;
                }

                Ok(())
            }
        }
    }
}
