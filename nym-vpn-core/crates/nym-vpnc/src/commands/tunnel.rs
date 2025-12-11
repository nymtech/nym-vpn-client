// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use nym_vpn_proto::rpc_client::RpcClient;

use crate::{boolean_option::BooleanOption, display_helpers::display_on_off};
use clap::builder::ValueParser;

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
    /// Set the average delay for a loop cover packet (milliseconds)
    #[arg(
        long,
        value_name = "MILLISECONDS",
        value_parser = ValueParser::from(|s: &str| -> Result<u32, String> {
            let val: u32 = s.parse().map_err(|_| format!("Invalid integer: {}", s))?;
            if !(0..=200).contains(&val) {
                return Err(format!("Value must be between 0 and 200 (got {val})"));
            }
            Ok(val)
        })
    )]
    pub loop_cover_stream_average_delay: Option<u32>,

    /// Set average packet delay at each mixnode (milliseconds)
    #[arg(
        long,
        value_name = "MILLISECONDS",
        value_parser = ValueParser::from(|s: &str| -> Result<u32, String> {
            let val: u32 = s.parse().map_err(|_| format!("Invalid integer: {}", s))?;
            if !(0..=200).contains(&val) {
                return Err(format!("Packet delay must be between 0 and 200 (got {val})"));
            }
            Ok(val)
        })
    )]
    pub average_packet_delay: Option<u32>,

    /// Set average real message sending delay (milliseconds)
    #[arg(
        long,
        value_name = "MILLISECONDS",
        value_parser = ValueParser::from(|s: &str| -> Result<u32, String> {
            let val: u32 = s.parse().map_err(|_| format!("Invalid integer: {}", s))?;
            if !(5..=50).contains(&val) {
                return Err(format!(
                    "Message sending delay must be between 5 and 50 (got {val})"
                ));
            }
            Ok(val)
        })
    )]
    pub message_sending_delay: Option<u32>,
    #[arg(
        long,
        help = "Disable Poisson process rate limiting for real traffic",
        value_parser = BooleanOption::custom_parser("on","off")
    )]
    pub disable_real_traffic_poisson_rate: Option<BooleanOption>,
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
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
                loop_cover_stream_average_delay,
                average_packet_delay,
                message_sending_delay,
                disable_real_traffic_poisson_rate,
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
                if let Some(poisson) = loop_cover_stream_average_delay {
                    rpc_client.set_poisson_parameter(poisson).await?;
                }
                if let Some(delay_ms) = average_packet_delay {
                    rpc_client.set_average_packet_delay(delay_ms).await?;
                }

                if let Some(delay_ms) = message_sending_delay {
                    rpc_client
                        .set_message_sending_average_delay(delay_ms)
                        .await?;
                }
                if let Some(disable) = disable_real_traffic_poisson_rate {
                    rpc_client.set_disable_poisson_rate(*disable).await?;
                }
                Ok(())
            }
        }
    }
}
