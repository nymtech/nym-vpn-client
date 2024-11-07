// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    str::FromStr,
    sync::OnceLock,
};

use clap::{Args, Parser, Subcommand};
use ipnetwork::{Ipv4Network, Ipv6Network};

const TUN_IP4_SUBNET: &str = "10.0.0.0/16";
const TUN_IP6_SUBNET: &str = "2001:db8:a160::0/112";

// Helper for passing LONG_VERSION to clap
fn pretty_build_info_static() -> &'static str {
    static PRETTY_BUILD_INFORMATION: OnceLock<String> = OnceLock::new();
    PRETTY_BUILD_INFORMATION.get_or_init(|| nym_bin_common::bin_info_local_vergen!().pretty_print())
}

#[derive(Parser)]
#[clap(author = "Nymtech", version, about, long_version = pretty_build_info_static())]
pub(crate) struct CliArgs {
    /// Path pointing to an env file describing the network.
    #[arg(short, long, value_parser = check_path)]
    pub(crate) config_env_file: Option<PathBuf>,

    /// Path to the data directory of the mixnet client.
    #[arg(long)]
    pub(crate) data_path: Option<PathBuf>,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Run the client
    Run(RunArgs),

    /// Store the account
    StoreAccount(StoreAccountArgs),
}

#[derive(Args)]
pub(crate) struct RunArgs {
    #[command(flatten)]
    pub(crate) entry: CliEntry,

    #[command(flatten)]
    pub(crate) exit: CliExit,

    /// Enable the wireguard mode.
    #[arg(long, default_value_t = false)]
    pub(crate) wireguard_mode: bool,

    /// Use wireguard with nestack for multihop.
    #[arg(long, default_value_t = false)]
    pub(crate) netstack: bool,

    /// The IPv4 address of the nym TUN device that wraps IP packets in sphinx packets.
    #[arg(long, alias = "ipv4", value_parser = validate_ipv4, requires = "nym_ipv6")]
    pub(crate) nym_ipv4: Option<Ipv4Addr>,

    /// The IPv6 address of the nym TUN device that wraps IP packets in sphinx packets.
    #[arg(long, alias = "ipv6", value_parser = validate_ipv6, requires = "nym_ipv4")]
    pub(crate) nym_ipv6: Option<Ipv6Addr>,

    /// The MTU of the nym TUN device that wraps IP packets in sphinx packets.
    #[arg(long, alias = "mtu")]
    pub(crate) nym_mtu: Option<u16>,

    /// The DNS server to use
    #[arg(long)]
    pub(crate) dns: Option<IpAddr>,

    /// Disable routing all traffic through the nym TUN device. When the flag is set, the nym TUN
    /// device will be created, but to route traffic through it you will need to do it manually,
    /// e.g. ping -Itun0.
    #[arg(long)]
    pub(crate) disable_routing: bool,

    /// Disable the Poisson process rate limiting of outbound traffic.
    #[arg(long, hide = true)]
    pub(crate) disable_poisson_rate: bool,

    /// Disable constant rate background loop cover traffic.
    #[arg(long, hide = true)]
    pub(crate) disable_background_cover_traffic: bool,

    /// Enable credentials mode.
    #[arg(long)]
    pub(crate) enable_credentials_mode: bool,

    /// Set the minimum performance level for mixnodes.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100), hide = true)]
    pub(crate) min_mixnode_performance: Option<u8>,

    // Set the minimum performance level for gateways.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub(crate) min_gateway_mixnet_performance: Option<u8>,

    // Set the minimum performance level for VPN gateways.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub(crate) min_gateway_vpn_performance: Option<u8>,
}

#[derive(Args)]
#[group(multiple = false)]
pub(crate) struct CliEntry {
    /// Mixnet public ID of the entry gateway.
    #[arg(long, alias = "entry-id")]
    pub(crate) entry_gateway_id: Option<String>,

    /// Auto-select entry gateway by country ISO.
    #[arg(long, alias = "entry-country")]
    pub(crate) entry_gateway_country: Option<String>,

    /// Auto-select entry gateway by latency
    #[arg(long, alias = "entry-fastest")]
    pub(crate) entry_gateway_low_latency: bool,
}

#[derive(Args)]
#[group(multiple = false)]
pub(crate) struct CliExit {
    /// Mixnet recipient address.
    #[arg(long, alias = "exit-address")]
    pub(crate) exit_router_address: Option<String>,

    /// Mixnet public ID of the exit gateway.
    #[arg(long, alias = "exit-id")]
    pub(crate) exit_gateway_id: Option<String>,

    /// Auto-select exit gateway by country ISO.
    #[arg(long, alias = "exit-country")]
    pub(crate) exit_gateway_country: Option<String>,
}

#[derive(Args)]
pub(crate) struct StoreAccountArgs {
    /// Recovery phrase for the account.
    #[arg(long)]
    pub(crate) mnemonic: String,
}

fn validate_ipv4(ip: &str) -> Result<Ipv4Addr, String> {
    let ip = Ipv4Addr::from_str(ip).map_err(|err| err.to_string())?;
    let network = Ipv4Network::from_str(TUN_IP4_SUBNET).unwrap();
    if !network.contains(ip) {
        return Err(format!("IPv4 address must be in the range {}", network));
    }
    if ip == Ipv4Addr::new(10, 0, 0, 1) {
        return Err("IPv4 address cannot be 10.0.0.1".to_string());
    }
    Ok(ip)
}

fn validate_ipv6(ip: &str) -> Result<Ipv6Addr, String> {
    let ip = Ipv6Addr::from_str(ip).map_err(|err| err.to_string())?;
    let network = Ipv6Network::from_str(TUN_IP6_SUBNET).unwrap();
    if !network.contains(ip) {
        return Err(format!("IPv6 address must be in the range {}", network));
    }
    if ip == Ipv6Addr::from_str("2001:db8:a160::1").unwrap() {
        return Err("IPv6 address cannot be 2001:db8:a160::1".to_string());
    }
    Ok(ip)
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
