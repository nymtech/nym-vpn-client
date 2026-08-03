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
        s.parse().map_err(|_| format!("Invalid integer: {}", s))
    })
    )]
    loop_cover_stream_average_delay: Option<u32>,

    /// Set average packet delay at each mixnode (milliseconds)
    #[arg(
    long,
    value_name = "MILLISECONDS",
    value_parser = ValueParser::from(|s: &str| -> Result<u32, String> {
        s.parse().map_err(|_| format!("Invalid integer: {}", s))
    })
    )]
    average_packet_delay: Option<u32>,

    /// Set average real message sending delay (milliseconds)
    #[arg(
    long,
    value_name = "MILLISECONDS",
    value_parser = ValueParser::from(|s: &str| -> Result<u32, String> {
        s.parse().map_err(|_| format!("Invalid integer: {}", s))
    })
    )]
    message_sending_delay: Option<u32>,
    #[arg(
        long,
        help = "Disable Poisson process rate limiting for real traffic",
        value_parser = clap::value_parser!(BooleanOption),
    )]
    disable_real_traffic_poisson_rate: Option<BooleanOption>,

    /// Enable or disable geo-location data being used for determining gateway proximity
    #[arg(long, value_parser = clap::value_parser!(BooleanOption))]
    geo_location: Option<BooleanOption>,

    /// Enable or disable gateway independence criteria when selecting the pair of servers
    #[arg(long, value_parser = clap::value_parser!(BooleanOption))]
    gateway_independence: Option<BooleanOption>,
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
                println!("Gateway independence: {}", config.gateway_independence);

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

                if let Some(enable_geo_location) = params.geo_location {
                    rpc_client
                        .set_enable_geo_location(*enable_geo_location)
                        .await?;
                }

                if let Some(gateway_independence) = params.gateway_independence {
                    rpc_client
                        .set_enable_gateway_independence(*gateway_independence)
                        .await?;
                }

                Ok(())
            }
        }
    }
}
