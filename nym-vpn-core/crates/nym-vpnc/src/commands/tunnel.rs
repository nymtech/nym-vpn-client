// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use nym_vpn_lib_types::{BridgeMode, BridgeTransport, VpnServiceConfig};
use nym_vpn_proto::rpc_client::RpcClient;

use crate::{
    boolean_option::BooleanOption,
    display_helpers::{self, display_on_off},
};
use clap::builder::ValueParser;

/// Transport preference, with an extra `auto` to clear back to no preference.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum TransportPreferenceArg {
    /// No preference: use whichever transport connects
    Auto,
    Quic,
    Tls,
    Ssh,
}

impl From<TransportPreferenceArg> for Option<BridgeTransport> {
    fn from(value: TransportPreferenceArg) -> Self {
        match value {
            TransportPreferenceArg::Auto => None,
            TransportPreferenceArg::Quic => Some(BridgeTransport::Quic),
            TransportPreferenceArg::Tls => Some(BridgeTransport::Tls),
            TransportPreferenceArg::Ssh => Some(BridgeTransport::Ssh),
        }
    }
}

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

    /// Set the bridge mode: auto tries plain WireGuard first and uses a bridge when that fails,
    /// on always uses a bridge, off never does.
    #[arg(long, value_enum)]
    bridge_mode: Option<BridgeMode>,

    /// Pin the bridge transport to use when bridging (ignored unless a bridge is in use); auto
    /// clears the pin and lets the client pick
    #[arg(long, value_enum)]
    transport_preference: Option<TransportPreferenceArg>,

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
                print_config(&config);
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

                if let Some(bridge_mode) = params.bridge_mode {
                    rpc_client.set_bridge_mode(bridge_mode).await?;
                }

                if let Some(transport_preference) = params.transport_preference {
                    rpc_client
                        .set_transport_preference(transport_preference.into())
                        .await?;
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

fn print_config(config: &VpnServiceConfig) {
    let mixnet_traffic = &config.mixnet_traffic;
    let gateway_independence = &config.gateway_independence;

    print_row("IPv6", display_on_off(!config.disable_ipv6));
    print_row("Two-hop", display_on_off(config.enable_two_hop));
    print_row("Netstack", display_on_off(config.netstack));
    print_row("Bridge mode", display_bridge_mode(config.bridge_mode));
    print_row(
        "Transport preference",
        display_transport_preference(config.transport_preference),
    );
    print_row(
        "Geo location",
        display_on_off(
            config
                .gateway_selection_algorithm_config
                .enable_geo_location,
        ),
    );
    print_row(
        "Gateway independence",
        display_on_off(gateway_independence.active()),
    );
    print_row(
        "  Different node family",
        display_on_off(gateway_independence.different_node_family),
    );
    print_row(
        "  Different ASN",
        display_on_off(gateway_independence.different_asn),
    );
    print_row(
        "  Different subnet",
        display_on_off(gateway_independence.different_subnet),
    );
    print_row(
        "  Notifications",
        display_on_off(gateway_independence.enable_notifications),
    );

    println!();
    println!("Mixnet traffic:");
    print_row(
        "  Loop cover stream delay",
        &display_optional(mixnet_traffic.poisson_parameter_for_loop_cover_stream, "ms"),
    );
    print_row(
        "  Average packet delay",
        &display_optional(mixnet_traffic.average_packet_delay, "ms"),
    );
    print_row(
        "  Message sending delay",
        &display_optional(mixnet_traffic.message_sending_average_delay, "ms"),
    );
    print_row(
        "  Real traffic Poisson rate",
        display_on_off(!mixnet_traffic.disable_poisson_rate),
    );
    print_row(
        "  Background cover traffic",
        display_on_off(!mixnet_traffic.disable_background_cover_traffic),
    );
    print_row(
        "  Min mixnode performance",
        &display_optional(mixnet_traffic.min_mixnode_performance, "%"),
    );
    print_row(
        "  Min gateway performance",
        &display_optional(mixnet_traffic.min_gateway_mixnet_performance, "%"),
    );
}

fn print_row(label: &str, value: &str) {
    display_helpers::print_row(29, label, value);
}

fn display_bridge_mode(bridge_mode: BridgeMode) -> &'static str {
    match bridge_mode {
        BridgeMode::Auto => "auto",
        BridgeMode::On => "on",
        BridgeMode::Off => "off",
    }
}

fn display_transport_preference(transport_preference: Option<BridgeTransport>) -> &'static str {
    match transport_preference {
        None => "auto",
        Some(BridgeTransport::Quic) => "quic",
        Some(BridgeTransport::Tls) => "tls",
        Some(BridgeTransport::Ssh) => "ssh",
    }
}

fn display_optional<T: std::fmt::Display>(value: Option<T>, unit: &str) -> String {
    value.map_or_else(|| "default".to_owned(), |value| format!("{value}{unit}"))
}
