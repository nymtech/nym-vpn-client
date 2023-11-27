// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

use clap::Parser;
use ipnetwork::{Ipv4Network};
use std::{
    net::{Ipv4Addr},
    path::PathBuf,
    str::FromStr,
};

const TUN_IP_SUBNET: &str = "10.0.0.0/24";

#[derive(Parser)]
#[clap(author = "Nymtech", version, about)]
pub(crate) struct CliArgs {
    /// Path pointing to an env file describing the network.
    #[arg(short, long)]
    pub(crate) config_env_file: Option<PathBuf>,

    /// Enable the wireguard traffic between the client and the entry gateway.
    #[arg(long, default_value_t = false, requires = "private_key")]
    pub(crate) enable_wireguard: bool,

    /// Path to the data directory of a previously initialised mixnet client, where the keys reside.
    #[arg(long)]
    pub(crate) mixnet_client_path: Option<PathBuf>,

    /// Mixnet public ID of the entry gateway.
    #[arg(long)]
    pub(crate) entry_gateway: String,

    /// Mixnet recipient address.
    #[arg(long, alias = "recipient-address", alias = "exit-address")]
    pub(crate) exit_router: String,

    /// Associated private key.
    #[arg(long, requires = "enable_wireguard")]
    pub(crate) private_key: Option<String>,

    /// The IP address of the TUN device.
    #[arg(long, value_parser = validate_ip)]
    pub(crate) ip: Ipv4Addr,

    /// Disable routing all traffic through the VPN TUN device.
    #[arg(long)]
    pub(crate) disable_routing: bool,
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

    return Ok(ip);
}
