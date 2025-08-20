// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use anyhow::{Result, anyhow};
use clap::{ArgAction, Args, Parser, Subcommand};
use nym_gateway_directory::{EntryPoint, ExitPoint, NodeIdentity, Recipient};
use nym_http_api_client::UserAgent;

#[derive(Parser)]
#[clap(author = "Nymtech", version, about)]
pub struct CliArgs {
    /// Override the default user agent string.
    #[arg(long, value_parser = parse_user_agent)]
    pub user_agent: Option<UserAgent>,

    #[command(subcommand)]
    pub command: Command,
}

fn parse_user_agent(user_agent: &str) -> Result<UserAgent> {
    Ok(UserAgent::try_from(user_agent)?)
}

#[derive(Subcommand)]
pub enum Command {
    /// Connect to the Nym network.
    Connect(Box<ConnectArgs>),

    /// Disconnect from the Nym network.
    Disconnect {
        /// Blocks until disconnected.
        #[arg(short, long, default_value = "false", action = ArgAction::SetTrue)]
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

    /// Get the current account controller state.
    GetAccountState {
        /// Monitor account controller state continuously until ctrl+c.
        #[arg(long, default_value = "false", action = ArgAction::SetTrue)]
        listen: bool,
    },

    /// Get URLs for managing your nym-vpn account.
    GetAccountLinks(GetAccountLinksArgs),

    /// Get the device ID.
    GetDeviceId,

    /// List the set of entry gateways for mixnet mode.
    ListEntryGateways,

    /// List the set of exit gateways for mixnet mode.
    ListExitGateways,

    /// List the set of entry and exit gateways for dVPN mode.
    ListVpnGateways,

    /// List the set of countries with available entry gateways for mixnet mode.
    ListEntryCountries,

    /// List the set of countries with available exit gateways for mixnet mode.
    ListExitCountries,

    /// List the set of countries with available entry and exit gateways for dVPN mode.
    ListVpnCountries,

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

    /// Get the devices associated with the account.
    GetDevices,

    /// Get the active devices associated with the account.
    GetActiveDevices,

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

    /// Disable IPv6 support
    #[arg(long)]
    pub disable_ipv6: bool,

    /// Enable two-hop wireguard traffic. This means that traffic jumps directly from entry gateway
    /// to exit gateway using Wireguard protocol.
    #[arg(long)]
    pub enable_two_hop: bool,

    /// Enable Circumvention Transport (CT) wrapping for the connection to the entry gateway in two
    /// hop wireguard mode.
    #[arg(long="ct", requires = "enable_two_hop")]
    pub circumvention_transports: bool,

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
}

impl ConnectArgs {
    pub fn entry_point(&self) -> Result<Option<EntryPoint>> {
        if let Some(ref entry_gateway_id) = self.entry.entry_id {
            Ok(Some(EntryPoint::Gateway {
                identity: NodeIdentity::from_base58_string(entry_gateway_id)
                    .map_err(|_| anyhow!("Failed to parse gateway id"))?,
            }))
        } else if let Some(ref entry_gateway_country) = self.entry.entry_country {
            Ok(Some(EntryPoint::Location {
                location: entry_gateway_country.alpha2.to_string(),
            }))
        } else if self.entry.entry_random {
            Ok(Some(EntryPoint::Random))
        } else {
            Ok(None)
        }
    }

    pub fn exit_point(&self) -> Result<Option<ExitPoint>> {
        if let Some(ref exit_router_address) = self.exit.exit_ipr_address {
            Ok(Some(ExitPoint::Address {
                address: Box::new(
                    Recipient::try_from_base58_string(exit_router_address)
                        .map_err(|_| anyhow!("Failed to parse exit node address"))?,
                ),
            }))
        } else if let Some(ref exit_router_id) = self.exit.exit_id {
            Ok(Some(ExitPoint::Gateway {
                identity: NodeIdentity::from_base58_string(exit_router_id.clone())
                    .map_err(|_| anyhow!("Failed to parse gateway id"))?,
            }))
        } else if let Some(ref exit_gateway_country) = self.exit.exit_country {
            Ok(Some(ExitPoint::Location {
                location: exit_gateway_country.alpha2.to_string(),
            }))
        } else if self.exit.exit_random {
            Ok(Some(ExitPoint::Random))
        } else {
            Ok(None)
        }
    }
}

#[derive(Args)]
#[group(multiple = false)]
pub struct CliEntry {
    /// Mixnet public ID of the entry gateway.
    #[arg(long, alias = "entry-gateway-id")]
    pub entry_id: Option<String>,

    /// Auto-select entry gateway by country ISO.
    #[arg(long, alias = "entry-gateway-country")]
    pub entry_country: Option<celes::Country>,

    /// Auto-select entry gateway randomly.
    #[arg(long, alias = "entry-gateway-random")]
    pub entry_random: bool,
}

#[derive(Args)]
#[group(multiple = false)]
pub struct CliExit {
    /// Mixnet recipient address of the IPR connecting to, if specified directly. This is only
    /// useful when connecting to standalone IPRs.
    #[clap(long, hide = true, alias = "exit-router-address")]
    pub exit_ipr_address: Option<String>,

    /// Mixnet public ID of the exit gateway.
    #[clap(long, alias = "exit-gateway-id")]
    pub exit_id: Option<String>,

    /// Auto-select exit gateway by country ISO.
    #[clap(long, alias = "exit-gateway-country")]
    pub exit_country: Option<celes::Country>,

    /// Auto-select exit gateway randomly.
    #[clap(long, alias = "exit-gateway-random")]
    pub exit_random: bool,
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
pub struct ListCountriesArgs {}

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
