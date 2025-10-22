// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use nym_vpn_proto::rpc_client::RpcClient;
use clap::builder::ValueParser;
use crate::boolean_option::BooleanOption;

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
    #[arg(long, value_parser = BooleanOption::custom_parser("on", "off"))]
    enable_ipv6: Option<BooleanOption>,

    /// Enable or disable two-hop mode
    #[arg(long, value_parser = BooleanOption::custom_parser("on", "off"))]
    enable_two_hop: Option<BooleanOption>,

    /// Enable or disable netstack in two-hop mode
    /// Normally this is only used for testing purposes and should always be off
    #[arg(long, value_parser = BooleanOption::custom_parser("on", "off"))]
    netstack: Option<BooleanOption>,

    /// Enable Circumvention Transport (CT) wrapping for the connection to the entry gateway in two hop wireguard mode.
    #[arg(long, alias = "ct", value_parser = BooleanOption::custom_parser("on", "off"))]
    circumvention_transports: Option<BooleanOption>,
    /// Set the average delay for a loop cover packet to be sent by the loop cover stream
    #[arg(
        long,
        value_name = "MILLISECONDS",
        value_parser = ValueParser::from(|s: &str| -> Result<f32, String> {
            let val: f32 = s.parse().map_err(|_| format!("Invalid float: {}", s))?;
            if !(0.0..=200.0).contains(&val) {
                return Err(format!("The must be between 0.0 and 200.0 (got {val})"));
            }
            Ok(val)
        })
    )]
    pub loop_cover_stream_average_delay: Option<f32>,
    /// Set average packet delay at each mixnode (milliseconds)
    #[arg(
        long,
        value_name = "MILLISECONDS",
        value_parser = ValueParser::from(|s: &str| -> Result<f32, String> {
            let val: f32 = s.parse().map_err(|_| format!("Invalid float: {}", s))?;
            if !(5.0..=50.0).contains(&val) {
                return Err(format!("Packet delay must be between 5.0 and 50.0 (got {val})"));
            }
            Ok(val)
        })
    )]
    pub average_packet_delay: Option<f32>,

    /// Set average real message sending delay (milliseconds). If no real messages are available,
    /// then dummy loop cover traffic packets are sent.
    #[arg(
        long,
        value_name = "MILLISECONDS",
        value_parser = ValueParser::from(|s: &str| -> Result<f32, String> {
            let val: f32 = s.parse().map_err(|_| format!("Invalid float: {}", s))?;
            if !(10.0..=20.0).contains(&val) {
                return Err(format!(
                    "Message sending delay must be between 10.0 and 20.0 (got {val})"
                ));
            }
            Ok(val)
        })
    )]
    pub message_sending_delay: Option<f32>,
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
                println!("IPv6: {}", if config.disable_ipv6 { "off" } else { "on" });
                println!(
                    "Two-hop: {}",
                    if config.enable_two_hop { "on" } else { "off" }
                );
                println!("Netstack: {}", if config.netstack { "on" } else { "off" });

                Ok(())
            }
            Command::Set(SetParams {
                enable_two_hop,
                netstack,
                enable_ipv6,
                circumvention_transports,
                loop_cover_stream_average_delay,
                average_packet_delay,
                message_sending_delay,
                disable_real_traffic_poisson_rate,
            }) => {
                if let Some(enable_two_hop) = enable_two_hop {
                    rpc_client.set_enable_two_hop(*enable_two_hop).await?;
                }

                if let Some(netstack) = netstack {
                    rpc_client.set_netstack(*netstack).await?;
                }

                if let Some(enable_ipv6) = enable_ipv6 {
                    rpc_client.set_disable_ipv6(!*enable_ipv6).await?;
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
                rpc_client.set_message_sending_average_delay(delay_ms).await?;
            }
            if let Some(disable) = disable_real_traffic_poisson_rate {
                rpc_client.set_disable_poisson_rate(*disable).await?;
            }
                Ok(())
            }
        }
    }
}
