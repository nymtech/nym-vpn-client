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
    Set(Box<SetParams>),
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
    loop_cover_stream_average_delay: Option<u32>,

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
    average_packet_delay: Option<u32>,

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
    message_sending_delay: Option<u32>,
    #[arg(
        long,
        help = "Disable Poisson process rate limiting for real traffic",
        value_parser = clap::value_parser!(BooleanOption),
    )]
    disable_real_traffic_poisson_rate: Option<BooleanOption>,
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
                println!("Mixnet traffic configuration: {}", config.mixnet_traffic);

                Ok(())
            }
            Command::Set(params) => {
                if let Some(two_hop) = params.two_hop {
                    rpc_client.set_enable_two_hop(*two_hop).await?;
                }

                if let Some(netstack) = params.netstack {
                    rpc_client.set_netstack(*netstack).await?;
                }

                if let Some(ipv6) = params.ipv6 {
                    rpc_client.set_disable_ipv6(!*ipv6).await?;
                }

                if let Some(enable_ct) = params.circumvention_transports {
                    rpc_client.set_enable_bridges(*enable_ct).await?;
                }

                if params.loop_cover_stream_average_delay.is_some()
                    || params.average_packet_delay.is_some()
                    || params.message_sending_delay.is_some()
                    || params.disable_real_traffic_poisson_rate.is_some()
                {
                    let mut config = rpc_client.get_config().await?;

                    if let Some(loop_delay) = params.loop_cover_stream_average_delay {
                        config
                            .mixnet_traffic
                            .poisson_parameter_for_loop_cover_stream = Some(loop_delay);
                    }

                    if let Some(average_packet_delay) = params.average_packet_delay {
                        config.mixnet_traffic.average_packet_delay = Some(average_packet_delay);
                    }

                    if let Some(message_sending_delay) = params.message_sending_delay {
                        config.mixnet_traffic.message_sending_average_delay =
                            Some(message_sending_delay);
                    }

                    if let Some(disable_poisson_rate) = params.disable_real_traffic_poisson_rate {
                        config.mixnet_traffic.disable_poisson_rate = *disable_poisson_rate;
                    }

                    rpc_client
                        .set_mixnet_traffic_config(config.mixnet_traffic)
                        .await?;
                }

                Ok(())
            }
        }
    }
}
