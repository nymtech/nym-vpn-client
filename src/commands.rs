// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(author = "Nymtech", version, about)]
pub(crate) struct CliArgs {
    /// Path pointing to an env file describing the network.
    #[clap(short, long)]
    pub(crate) config_env_file: Option<PathBuf>,

    /// Enable the wireguard traffic between the client and the entry gateway.
    #[clap(long, default_value_t = false)]
    pub(crate) enable_wireguard: bool,

    /// Path to the data directory of a previously initialised mixnet client, where the keys reside.
    #[clap(long)]
    pub(crate) mixnet_client_path: PathBuf,

    /// Mixnet public ID of the entry gateway.
    #[clap(long)]
    pub(crate) entry_gateway: String,

    /// Mixnet recipient address.
    #[clap(long)]
    pub(crate) recipient_address: String,

    /// Associated private key.
    #[clap(long)]
    pub(crate) private_key: String,

    /// Preshared key (PSK).
    #[clap(long)]
    pub(crate) psk: Option<String>,
}
