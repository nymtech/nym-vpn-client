// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

use clap::Parser;
use ipnetwork::Ipv4Network;
use nym_config::defaults::var_names::NYM_API;
use nym_config::OptionalSet;
use nym_vpn_cli::gateway_client::Config;
use std::{net::Ipv4Addr, path::PathBuf, str::FromStr};

const TUN_IP_SUBNET: &str = "10.0.0.0/24";

#[derive(Parser)]
#[clap(author = "Nymtech", version, about)]
pub(crate) struct CliArgs {
    /// Path pointing to an env file describing the network.
    #[arg(short, long)]
    pub(crate) config_env_file: Option<PathBuf>,

    /// Path to the data directory of a previously initialised mixnet client, where the keys reside.
    #[arg(long)]
    pub(crate) mixnet_client_path: Option<PathBuf>,

    /// Mixnet public ID of the entry gateway.
    #[arg(long)]
    pub(crate) entry_gateway: String,

    /// Mixnet recipient address.
    #[arg(long, alias = "recipient-address", alias = "exit-address")]
    pub(crate) exit_router: String,

    /// Enable the wireguard traffic between the client and the entry gateway.
    #[arg(long, default_value_t = false, requires = "private_key")]
    pub(crate) enable_wireguard: bool,

    /// Associated private key.
    #[arg(long, requires = "enable_wireguard")]
    pub(crate) private_key: Option<String>,

    /// The IP address of the TUN device.
    #[arg(long, value_parser = validate_ip)]
    pub(crate) ip: Option<Ipv4Addr>,

    /// The MTU of the TUN device.
    #[arg(long)]
    pub(crate) mtu: Option<i32>,

    /// Disable routing all traffic through the VPN TUN device.
    #[arg(long)]
    pub(crate) disable_routing: bool,

    /// Enable two-hop mixnet traffic. This means that traffic jumps directly from entry gateway to
    /// exit gateway.
    #[arg(long)]
    pub(crate) enable_two_hop: bool,

    /// Enable Poission process rate limiting of outbound traffic.
    #[arg(long)]
    pub(crate) enable_poisson_rate: bool,
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
