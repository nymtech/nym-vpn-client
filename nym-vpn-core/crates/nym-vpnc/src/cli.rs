// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use anyhow::{anyhow, Result};
use clap::{ArgAction, Args, Parser, Subcommand};
use nym_gateway_directory::{EntryPoint, ExitPoint, NodeIdentity, Recipient};

#[derive(Parser)]
#[clap(author = "Nymtech", version, about)]
pub struct CliArgs {
    /// Use HTTP instead of socket file for IPC with the daemon.
    #[arg(long)]
    pub http: bool,

    #[arg(long)]
    pub verbose: bool,

    /// Override the default user agent string.
    #[arg(long, value_parser = parse_user_agent)]
    pub user_agent: Option<nym_http_api_client::UserAgent>,

    #[command(subcommand)]
    pub command: Command,
}

// TODO: make use of the From<&str> implementation for UserAgent once that is available in the
// upstream branch.
fn parse_user_agent(user_agent: &str) -> Result<nym_http_api_client::UserAgent, String> {
    let parts: Vec<&str> = user_agent.split('/').collect();
    if parts.len() != 4 {
        return Err("User agent must have 4 parts".to_string());
    }

    Ok(nym_http_api_client::UserAgent {
        application: parts[0].to_string(),
        version: parts[1].to_string(),
        platform: parts[2].to_string(),
        git_commit: parts[3].to_string(),
    })
}

#[derive(Subcommand)]
pub enum Command {
    /// Connect to the Nym network.
    Connect(ConnectArgs),

    /// Disconnect from the Nym network.
    Disconnect {
        /// Blocks until disconnected.
        #[arg(long, default_value = "false", action = ArgAction::SetTrue)]
        wait: bool,
    },

    /// Get the current status of the connection.
    Status {
        /// Monitor tunnel state continuously until ctrl+c.
        #[arg(long, default_value = "false", action = ArgAction::SetTrue)]
        listen: bool,
    },

    /// Get info about the current client. Things like version and network details.
    Info,

    /// Set the network to be used. This requires a restart of the daemon (`nym-vpnd`)
    SetNetwork(SetNetworkArgs),

    /// Store the account recovery phrase.
    StoreAccount(StoreAccountArgs),

    /// Check if the account is stored.
    IsAccountStored,

    /// Forget the stored account. This removes the stores recovery phrase, device and mixnet keys,
    /// stored local credentials, etc.
    ForgetAccount,

    /// Get the account ID.
    GetAccountId,

    /// Get the current account state.
    GetAccountState,

    /// Get URLs for managing your nym-vpn account.
    GetAccountLinks(GetAccountLinksArgs),

    /// Get the device ID.
    GetDeviceId,

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

    /// Internal commands for development and debugging.
    #[clap(subcommand, hide = true)]
    Internal(Internal),
}

#[derive(Subcommand)]
pub enum Internal {
    /// Get the list of system messages provided by the nym-vpn-api.
    GetSystemMessages,

    /// Get the list of feature flags provided by the nym-vpn-api.
    GetFeatureFlags,

    /// Manually trigger an account sync with the nym-vpn-api.
    SyncAccountState,

    /// Get the account usage from the nym-vpn-api.
    GetAccountUsage,

    /// Manually reset the device identity. A seed can be provided as a way to generate a stable
    /// identity for testing.
    ResetDeviceIdentity(ResetDeviceIdentityArgs),

    /// Register the device with your account.
    RegisterDevice,

    /// Get the devices associated with the account.
    GetDevices,

    /// Get the active devices associated with the account.
    GetActiveDevices,

    /// Manually request zknym credentials.
    RequestZkNym,

    /// Get the zknym credentials associated with this device.
    GetDeviceZkNym,

    /// Get the zknym credentials available to download for this device.
    GetZkNymsAvailableForDownload,

    /// Get a specific zknym credential by ID.
    GetZkNymById(GetZkNymByIdArgs),

    /// Manually confirm that a zknym credential has been downloaded to the device and stored in
    /// the local credential store.
    ConfirmZkNymDownloaded(ConfirmZkNymDownloadedArgs),

    /// List the available zknym ticketbooks in the local credential store.
    GetAvailableTickets,
}

#[derive(Args)]
pub struct ConnectArgs {
    #[command(flatten)]
    pub entry: CliEntry,

    #[command(flatten)]
    pub exit: CliExit,

    /// Set the IP address of the DNS server to use.
    #[arg(long)]
    pub dns: Option<IpAddr>,

    /// Enable two-hop wireguard traffic. This means that traffic jumps directly from entry gateway to
    /// exit gateway using Wireguard protocol.
    #[arg(long)]
    pub enable_two_hop: bool,

    /// Blocks until the connection is established or failed
    #[arg(short, long)]
    pub wait: bool,

    /// Use netstack based implementation for two-hop wireguard.
    #[arg(long, requires = "enable_two_hop")]
    pub netstack: bool,

    /// Disable Poisson process rate limiting of outbound traffic.
    #[arg(long, hide = true)]
    pub disable_poisson_rate: bool,

    /// Disable constant rate background loop cover traffic.
    #[arg(long, hide = true)]
    pub disable_background_cover_traffic: bool,

    /// Enable credentials mode.
    #[arg(long)]
    pub enable_credentials_mode: bool,

    /// An integer between 0 and 100 representing the minimum mixnode performance required to
    /// consider a mixnode for routing traffic.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100), hide = true)]
    pub min_mixnode_performance: Option<u8>,

    /// An integer between 0 and 100 representing the minimum gateway performance required to
    /// consider a gateway for routing traffic.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub min_gateway_mixnet_performance: Option<u8>,

    /// An integer between 0 and 100 representing the minimum gateway performance required to
    /// consider a gateway for routing traffic.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub min_gateway_vpn_performance: Option<u8>,
}

#[derive(Args)]
#[group(multiple = false)]
pub struct CliEntry {
    /// Mixnet public ID of the entry gateway.
    #[arg(long, alias = "entry-id")]
    pub entry_gateway_id: Option<String>,

    /// Auto-select entry gateway by country ISO.
    #[arg(long, alias = "entry-country")]
    pub entry_gateway_country: Option<String>,

    /// Auto-select entry gateway by latency
    #[arg(long, alias = "entry-fastest")]
    pub entry_gateway_low_latency: bool,

    /// Auto-select entry gateway randomly.
    #[arg(long, alias = "entry-random")]
    pub entry_gateway_random: bool,
}

#[derive(Args)]
#[group(multiple = false)]
pub struct CliExit {
    /// Mixnet recipient address.
    #[clap(long, alias = "exit-address")]
    pub exit_router_address: Option<String>,

    /// Mixnet public ID of the exit gateway.
    #[clap(long, alias = "exit-id")]
    pub exit_gateway_id: Option<String>,

    /// Auto-select exit gateway by country ISO.
    #[clap(long, alias = "exit-country")]
    pub exit_gateway_country: Option<String>,

    /// Auto-select exit gateway randomly.
    #[clap(long, alias = "exit-random")]
    pub exit_gateway_random: bool,
}

#[derive(Args)]
pub struct SetNetworkArgs {
    /// The network to be set.
    pub network: String,
}

#[derive(Args)]
pub struct StoreAccountArgs {
    /// The account mnemonic to be stored.
    #[arg(long)]
    pub mnemonic: String,
}

#[derive(Args)]
pub struct GetAccountLinksArgs {
    /// The locale to be used.
    #[arg(long)]
    pub locale: String,
}

#[derive(Args)]
pub struct ListGatewaysArgs {
    /// Display additional information about the gateways.
    #[arg(long, short)]
    pub verbose: bool,

    /// An integer between 0 and 100 representing the minimum gateway performance required to
    /// consider a gateway for routing traffic.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub min_mixnet_performance: Option<u8>,

    /// An integer between 0 and 100 representing the minimum gateway performance required to
    /// consider a gateway for routing traffic.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub min_vpn_performance: Option<u8>,
}

#[derive(Args)]
pub struct ListCountriesArgs {
    /// An integer between 0 and 100 representing the minimum gateway performance required to
    /// consider a gateway for routing traffic.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub min_mixnet_performance: Option<u8>,

    /// An integer between 0 and 100 representing the minimum gateway performance required to
    /// consider a gateway for routing traffic.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub min_vpn_performance: Option<u8>,
}

#[derive(Args)]
pub struct ResetDeviceIdentityArgs {
    /// Reset the device identity using the given seed.
    #[arg(long)]
    pub seed: Option<String>,
}

#[derive(Args)]
pub struct GetZkNymByIdArgs {
    /// The ID of the ZK Nym to fetch.
    #[arg(short, long)]
    pub id: String,
}

#[derive(Args)]
pub struct ConfirmZkNymDownloadedArgs {
    /// The ID of the ZK Nym to confirm.
    #[arg(short, long)]
    pub id: String,
}

pub fn parse_entry_point(args: &ConnectArgs) -> Result<Option<EntryPoint>> {
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

pub fn parse_exit_point(args: &ConnectArgs) -> Result<Option<ExitPoint>> {
    if let Some(ref exit_router_address) = args.exit.exit_router_address {
        Ok(Some(ExitPoint::Address {
            address: Box::new(
                Recipient::try_from_base58_string(exit_router_address.clone())
                    .map_err(|_| anyhow!("Failed to parse exit node address"))?,
            ),
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
