// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use clap::Parser;
use ipnetwork::Ipv4Network;
use nym_vpn_lib::nym_config::defaults::var_names::NYM_API;
use nym_vpn_lib::nym_config::OptionalSet;
use nym_vpn_lib::{gateway_client::Config, nym_bin_common::bin_info};
use std::{net::Ipv4Addr, path::PathBuf, str::FromStr, sync::OnceLock};

const WG_IP_SUBNET: &str = "10.1.0.0/16";
const TUN_IP_SUBNET: &str = "10.0.0.0/24";

// Helper for passing LONG_VERSION to clap
fn pretty_build_info_static() -> &'static str {
    static PRETTY_BUILD_INFORMATION: OnceLock<String> = OnceLock::new();
    PRETTY_BUILD_INFORMATION.get_or_init(|| bin_info!().pretty_print())
}

#[derive(Parser)]
#[clap(author = "Nymtech", version, about, long_version = pretty_build_info_static())]
pub(crate) struct CliArgs {
    /// Path pointing to an env file describing the network.
    #[arg(short, long, value_parser = check_path)]
    pub(crate) config_env_file: Option<PathBuf>,

    /// Path to the data directory of a previously initialised mixnet client, where the keys reside.
    #[arg(long)]
    pub(crate) mixnet_client_path: Option<PathBuf>,

    /// Mixnet public ID of the entry gateway.
    #[clap(long, conflicts_with = "entry_gateway_country", alias = "entry-id")]
    pub(crate) entry_gateway_id: Option<String>,

    /// Auto-select entry gateway by country ISO.
    #[clap(long, conflicts_with = "entry_gateway_id", alias = "entry-country")]
    pub(crate) entry_gateway_country: Option<String>,

    /// Mixnet recipient address.
    #[arg(
        long,
        conflicts_with = "exit_router_country",
        conflicts_with = "exit_gateway_id",
        alias = "exit-address"
    )]
    pub(crate) exit_router_address: Option<String>,

    #[clap(
        long,
        conflicts_with = "exit_router_country",
        conflicts_with = "exit_router_address",
        alias = "exit-id"
    )]
    pub(crate) exit_gateway_id: Option<String>,

    /// Mixnet recipient address.
    #[arg(
        long,
        alias = "exit-country",
        conflicts_with = "exit_router_address",
        conflicts_with = "exit_gateway_id"
    )]
    pub(crate) exit_router_country: Option<String>,

    /// Enable the wireguard traffic between the client and the entry gateway.
    #[arg(
        long,
        default_value_t = false,
        requires = "private_key",
        requires = "wg_ip"
    )]
    pub(crate) enable_wireguard: bool,

    /// Associated private key.
    #[arg(long, requires = "enable_wireguard", requires = "wg_ip")]
    pub(crate) private_key: Option<String>,

    /// The IP address of the wireguard interface used for the first hop to the entry gateway.
    #[arg(long, value_parser = validate_wg_ip, requires = "enable_wireguard")]
    pub(crate) wg_ip: Option<Ipv4Addr>,

    /// The IP address of the nym TUN device that wraps IP packets in sphinx packets.
    #[arg(long, alias = "ip", value_parser = validate_ip)]
    pub(crate) nym_ip: Option<Ipv4Addr>,

    /// The MTU of the nym TUN device that wraps IP packets in sphinx packets.
    #[arg(long, alias = "mtu")]
    pub(crate) nym_mtu: Option<usize>,

    /// Disable routing all traffic through the nym TUN device. When the flag is set, the nym TUN
    /// device will be created, but to route traffic through it you will need to do it manually,
    /// e.g. ping -Itun0.
    #[arg(long)]
    pub(crate) disable_routing: bool,

    /// Enable two-hop mixnet traffic. This means that traffic jumps directly from entry gateway to
    /// exit gateway.
    #[arg(long)]
    pub(crate) enable_two_hop: bool,

    /// Enable Poisson process rate limiting of outbound traffic.
    #[arg(long)]
    pub(crate) enable_poisson_rate: bool,

    /// Disable constant rate background loop cover traffic
    #[arg(long)]
    pub(crate) disable_background_cover_traffic: bool,
}

fn validate_wg_ip(ip: &str) -> Result<Ipv4Addr, String> {
    let ip = Ipv4Addr::from_str(ip).map_err(|err| err.to_string())?;
    let network = Ipv4Network::from_str(WG_IP_SUBNET).unwrap();
    if !network.contains(ip) {
        return Err(format!("IP address must be in the range {}", network));
    }
    if ip == Ipv4Addr::new(10, 1, 0, 1) {
        return Err("IP address cannot be 10.1.0.1".to_string());
    }
    Ok(ip)
}

fn validate_ip(ip: &str) -> Result<Ipv4Addr, String> {
    let ip = Ipv4Addr::from_str(ip).map_err(|err| err.to_string())?;
    let network = Ipv4Network::from_str(TUN_IP_SUBNET).unwrap();
    if !network.contains(ip) {
        return Err(format!("IP address must be in the range {}", network));
    }
    if ip == Ipv4Addr::new(10, 0, 0, 1) {
        return Err("IP address cannot be 10.0.0.1".to_string());
    }
    Ok(ip)
}

pub fn override_from_env(args: &CliArgs, config: Config) -> Config {
    let mut config = config.with_optional_env(Config::with_custom_api_url, None, NYM_API);
    if let Some(ref private_key) = args.private_key {
        config = config.with_local_private_key(private_key.clone());
    }
    config
}

fn check_path(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(format!("Path {:?} does not exist", path));
    }
    if !path.is_file() {
        return Err(format!("Path {:?} is not a file", path));
    }
    Ok(path)
}
