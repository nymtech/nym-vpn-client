// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use nym_gateway_directory::{EntryPoint, ExitPoint, NodeIdentity, Recipient};

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
    /// Connect to the Nym network.
    Connect(ConnectArgs),
    /// Disconnect from the Nym network.
    Disconnect,
    /// Get the current status of the connection.
    Status,
    /// Get info about the current client. Things like version and network details.
    Info,
    /// Set the network to be used. This requires a restart of the daemon (`nym-vpnd`)
    SetNetwork(SetNetworkArgs),
    #[clap(hide = true)]
    GetSystemMessages,
    #[clap(hide = true)]
    GetFeatureFlags,
    /// Store the account recovery phrase.
    StoreAccount(StoreAccountArgs),
    /// Check if the account is stored.
    IsAccountStored,
    /// Remove the stored account. This only removes the stored recovery phrase.
    #[clap(hide = true)]
    RemoveAccount,
    /// Forget the stored account. This removes the stores recovery phrase, device and mixnet keys,
    /// stored local credentials, etc.
    ForgetAccount,
    /// Get the account ID.
    GetAccountId,
    /// Get the current account state.
    GetAccountState,
    /// Get URLs for managing your nym-vpn account.
    GetAccountLinks(GetAccountLinksArgs),
    #[clap(hide = true)]
    RefreshAccountState,
    #[clap(hide = true)]
    IsReadyToConnect,
    #[clap(hide = true)]
    ResetDeviceIdentity(ResetDeviceIdentityArgs),
    /// Get the device ID.
    GetDeviceId,
    /// Register the device with your account.
    #[clap(hide = true)]
    RegisterDevice,
    /// Request zknym credentials.
    #[clap(hide = true)]
    RequestZkNym,
    #[clap(hide = true)]
    GetDeviceZkNym,
    #[clap(hide = true)]
    GetZkNymsAvailableForDownload,
    #[clap(hide = true)]
    GetZkNymById(GetZkNymByIdArgs),
    #[clap(hide = true)]
    ConfirmZkNymDownloaded(ConfirmZkNymDownloadedArgs),
    #[clap(hide = true)]
    GetAvailableTickets,
    #[clap(hide = true)]
    FetchRawAccountSummary,
    #[clap(hide = true)]
    FetchRawDevices,
    #[clap(hide = true)]
    ListenToStatus,
    #[clap(hide = true)]
    ListenToStateChanges,
    /// List the set of entry gateways for mixnet mode.
    ListEntryGateways(ListGatewaysArgs),
    /// List the set of exit gateways for mixnet mode.
    ListExitGateways(ListGatewaysArgs),
    /// List the set of entry and exit gateways for dVPN mode.
    ListVpnGateways(ListGatewaysArgs),
    /// List the set of countries with available entry gateways for mixnet mode.
    ListEntryCountries(ListCountriesArgs),
    /// List the set of countries with available exit gateways for mixnet mode.
    ListExitCountries(ListCountriesArgs),
    /// List the set of countries with available entry and exit gateways for dVPN mode.
    ListVpnCountries(ListCountriesArgs),
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

    /// Enable two-hop wireguard traffic. This means that traffic jumps directly from entry gateway to
    /// exit gateway using Wireguard protocol.
    #[arg(long)]
    pub(crate) enable_two_hop: bool,

    /// Disable Poisson process rate limiting of outbound traffic.
    #[arg(long, hide = true)]
    pub(crate) disable_poisson_rate: bool,

    /// Disable constant rate background loop cover traffic.
    #[arg(long, hide = true)]
    pub(crate) disable_background_cover_traffic: bool,

    /// Enable credentials mode.
    #[arg(long)]
    pub(crate) enable_credentials_mode: bool,

    /// An integer between 0 and 100 representing the minimum mixnode performance required to
    /// consider a mixnode for routing traffic.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100), hide = true)]
    pub(crate) min_mixnode_performance: Option<u8>,

    /// An integer between 0 and 100 representing the minimum gateway performance required to
    /// consider a gateway for routing traffic.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub(crate) min_gateway_mixnet_performance: Option<u8>,

    /// An integer between 0 and 100 representing the minimum gateway performance required to
    /// consider a gateway for routing traffic.
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

    /// Auto-select entry gateway randomly.
    #[arg(long, alias = "entry-random")]
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
pub(crate) struct SetNetworkArgs {
    /// The network to be set.
    pub(crate) network: String,
}

#[derive(Args)]
pub(crate) struct StoreAccountArgs {
    /// The account mnemonic to be stored.
    #[arg(long)]
    pub(crate) mnemonic: String,
}

#[derive(Args)]
pub(crate) struct GetAccountLinksArgs {
    /// The locale to be used.
    #[arg(long)]
    pub(crate) locale: String,
}

#[derive(Args)]
pub(crate) struct ListGatewaysArgs {
    /// Display additional information about the gateways.
    #[arg(long, short)]
    pub(crate) verbose: bool,

    /// An integer between 0 and 100 representing the minimum gateway performance required to
    /// consider a gateway for routing traffic.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub(crate) min_mixnet_performance: Option<u8>,

    /// An integer between 0 and 100 representing the minimum gateway performance required to
    /// consider a gateway for routing traffic.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub(crate) min_vpn_performance: Option<u8>,
}

#[derive(Args)]
pub(crate) struct ListCountriesArgs {
    /// An integer between 0 and 100 representing the minimum gateway performance required to
    /// consider a gateway for routing traffic.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub(crate) min_mixnet_performance: Option<u8>,

    /// An integer between 0 and 100 representing the minimum gateway performance required to
    /// consider a gateway for routing traffic.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub(crate) min_vpn_performance: Option<u8>,
}

#[derive(Args)]
pub(crate) struct ResetDeviceIdentityArgs {
    /// Reset the device identity using the given seed.
    #[arg(long)]
    pub(crate) seed: Option<String>,
}

#[derive(Args)]
pub(crate) struct GetZkNymByIdArgs {
    /// The ID of the ZK Nym to fetch.
    #[arg(short, long)]
    pub(crate) id: String,
}

#[derive(Args)]
pub(crate) struct ConfirmZkNymDownloadedArgs {
    /// The ID of the ZK Nym to confirm.
    #[arg(short, long)]
    pub(crate) id: String,
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
