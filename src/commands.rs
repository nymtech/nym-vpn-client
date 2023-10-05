// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

use clap::Parser;

#[derive(Parser)]
#[clap(author = "Nymtech", version, about)]
pub(crate) struct CliArgs {
    /// Associated private key.
    #[clap(long)]
    pub(crate) private_key: String,

    /// Local IP addresses associated with a key pair.
    #[clap(long, num_args = 1.., value_delimiter = ' ')]
    pub(crate) addresses: Vec<String>,

    /// Peer's public key.
    #[clap(long)]
    pub(crate) public_key: String,

    /// Addresses that may be routed to the peer. Use `0.0.0.0/0` to route everything.
    #[clap(long, num_args = 1.., value_delimiter = ' ')]
    pub(crate) allowed_ips: Vec<String>,

    /// IP address of the WireGuard server.
    #[clap(long)]
    pub(crate) endpoint: String,

    /// Preshared key (PSK).
    #[clap(long)]
    pub(crate) psk: Option<String>,

    /// IPv4 gateway.
    #[clap(long)]
    pub(crate) ipv4_gateway: String,
}
