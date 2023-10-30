// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

use clap::Parser;

#[derive(Parser)]
#[clap(author = "Nymtech", version, about)]
pub(crate) struct CliArgs {
    /// Path pointing to an env file describing the network.
    #[clap(short, long)]
    pub(crate) config_env_file: Option<std::path::PathBuf>,

    /// Mixnet public ID of the entry gateway.
    #[clap(long)]
    pub(crate) entry_gateway: String,

    /// Mixnet recipient address.
    #[clap(long)]
    pub(crate) recipient_address: String,

    /// Associated private key.
    #[clap(long)]
    pub(crate) private_key: String,

    /// Local IP addresses associated with a key pair.
    #[clap(long, num_args = 1.., value_delimiter = ' ')]
    pub(crate) addresses: Vec<String>,

    /// Preshared key (PSK).
    #[clap(long)]
    pub(crate) psk: Option<String>,

    /// IPv4 gateway.
    #[clap(long)]
    pub(crate) ipv4_gateway: String,
}
