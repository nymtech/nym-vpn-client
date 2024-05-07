// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use nym_gateway_directory::{EntryPoint, ExitPoint, NodeIdentity, Recipient};
use std::{net::IpAddr, path::PathBuf};

#[derive(Parser)]
#[clap(author = "Nymtech", version, about)]
pub(crate) struct CliArgs {
    /// Use HTTP instead of socket file for IPC with the daemon.
    #[arg(long)]
    pub(crate) http: bool,

    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    Connect(ConnectArgs),
    Disconnect,
    Status,
    ImportCredential(ImportCredentialArgs),
    ListenToStatus,
}

#[derive(Args)]
pub(crate) struct ConnectArgs {
    #[command(flatten)]
    pub(crate) entry: CliEntry,

    #[command(flatten)]
    pub(crate) exit: CliExit,

    /// Set the IP address of the DNS server to use.
    #[arg(long)]
    pub(crate) dns: Option<IpAddr>,

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

    /// Disable constant rate background loop cover traffic.
    #[arg(long)]
    pub(crate) disable_background_cover_traffic: bool,

    /// Enable credentials mode.
    #[arg(long)]
    pub(crate) enable_credentials_mode: bool,
}

#[derive(Args)]
#[group(multiple = false)]
pub(crate) struct CliEntry {
    /// Mixnet public ID of the entry gateway.
    #[clap(long, alias = "entry-id")]
    pub(crate) entry_gateway_id: Option<String>,

    /// Auto-select entry gateway by country ISO.
    #[clap(long, alias = "entry-country")]
    pub(crate) entry_gateway_country: Option<String>,

    /// Auto-select entry gateway by latency
    #[clap(long, alias = "entry-fastest")]
    pub(crate) entry_gateway_low_latency: bool,

    /// Auto-select entry gateway randomly.
    #[clap(long, alias = "entry-random")]
    pub(crate) entry_gateway_random: bool,
}

#[derive(Args)]
#[group(multiple = false)]
pub(crate) struct CliExit {
    /// Mixnet recipient address.
    #[clap(long, alias = "exit-address")]
    pub(crate) exit_router_address: Option<String>,

    /// Mixnet public ID of the exit gateway.
    #[clap(long, alias = "exit-id")]
    pub(crate) exit_gateway_id: Option<String>,

    /// Auto-select exit gateway by country ISO.
    #[clap(long, alias = "exit-country")]
    pub(crate) exit_gateway_country: Option<String>,

    /// Auto-select exit gateway randomly.
    #[clap(long, alias = "exit-random")]
    pub(crate) exit_gateway_random: bool,
}

#[derive(Args)]
pub(crate) struct ImportCredentialArgs {
    #[command(flatten)]
    pub(crate) credential_type: ImportCredentialType,

    // currently hidden as there exists only a single serialization standard
    #[arg(long, hide = true)]
    pub(crate) version: Option<u8>,
}

#[derive(Args, Clone)]
#[group(required = true, multiple = false)]
pub(crate) struct ImportCredentialType {
    /// Credential encoded using base58.
    #[arg(long)]
    pub(crate) credential_data: Option<String>,

    /// Path to the credential file.
    #[arg(long)]
    pub(crate) credential_path: Option<PathBuf>,
}

// Workaround until clap supports enums for ArgGroups
pub(crate) enum ImportCredentialTypeEnum {
    Path(PathBuf),
    Data(String),
}

impl From<ImportCredentialType> for ImportCredentialTypeEnum {
    fn from(ict: ImportCredentialType) -> Self {
        match (ict.credential_data, ict.credential_path) {
            (Some(data), None) => ImportCredentialTypeEnum::Data(data),
            (None, Some(path)) => ImportCredentialTypeEnum::Path(path),
            _ => unreachable!(),
        }
    }
}

pub(crate) fn parse_entry_point(args: &ConnectArgs) -> Result<Option<EntryPoint>> {
    if let Some(ref entry_gateway_id) = args.entry.entry_gateway_id {
        Ok(Some(EntryPoint::Gateway {
            identity: NodeIdentity::from_base58_string(entry_gateway_id.clone())
                .map_err(|_| anyhow!("Failed to parse gateway id"))?,
        }))
    } else if let Some(ref entry_gateway_country) = args.entry.entry_gateway_country {
        Ok(Some(EntryPoint::Location {
            location: entry_gateway_country.clone(),
        }))
    } else if args.entry.entry_gateway_low_latency {
        Ok(Some(EntryPoint::RandomLowLatency))
    } else if args.entry.entry_gateway_random {
        Ok(Some(EntryPoint::Random))
    } else {
        Ok(None)
    }
}

pub(crate) fn parse_exit_point(args: &ConnectArgs) -> Result<Option<ExitPoint>> {
    if let Some(ref exit_router_address) = args.exit.exit_router_address {
        Ok(Some(ExitPoint::Address {
            address: Recipient::try_from_base58_string(exit_router_address.clone())
                .map_err(|_| anyhow!("Failed to parse exit node address"))?,
        }))
    } else if let Some(ref exit_router_id) = args.exit.exit_gateway_id {
        Ok(Some(ExitPoint::Gateway {
            identity: NodeIdentity::from_base58_string(exit_router_id.clone())
                .map_err(|_| anyhow!("Failed to parse gateway id"))?,
        }))
    } else if let Some(ref exit_gateway_country) = args.exit.exit_gateway_country {
        Ok(Some(ExitPoint::Location {
            location: exit_gateway_country.clone(),
        }))
    } else if args.exit.exit_gateway_random {
        Ok(Some(ExitPoint::Random))
    } else {
        Ok(None)
    }
}
