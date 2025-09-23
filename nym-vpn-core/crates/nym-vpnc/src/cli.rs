// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::IpAddr, ops::Deref};

use anyhow::{Result, anyhow};
use clap::{
    ArgAction, Args, Parser, Subcommand,
    builder::{PossibleValuesParser, TypedValueParser, ValueParser, ValueParserFactory},
};
use nym_gateway_directory::{EntryPoint, ExitPoint, NodeIdentity, Recipient};
use nym_http_api_client::UserAgent;

#[derive(Parser)]
#[clap(version, about)]
pub struct LegacyCliArgs {
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
    /// Connect to the Nym network (deprecated)
    Connect(Box<ConnectArgs>),

    /// Connect the tunnel if it had been disconnected
    ConnectV2 {
        /// Blocks until the connection is established or failed
        #[arg(short, long)]
        wait: bool,
    },

    /// Reconnect the tunnel if it had been connected
    Reconnect,

    /// Disconnect the tunnel
    Disconnect {
        /// Blocks until disconnected.
        #[arg(short, long, default_value = "false", action = ArgAction::SetTrue)]
        wait: bool,
    },

    /// Get the current connection status
    Status {
        /// Monitor tunnel state continuously until ctrl+c.
        #[arg(long, default_value = "false", action = ArgAction::SetTrue)]
        listen: bool,
    },

    /// Get info about the current client. Things like version and network details.
    Info,

    /// Get the current VPN service configuration.
    GetConfig,

    /// Set the entry gateway node
    SetEntry {
        #[command(flatten)]
        entry: CliEntry,
    },

    /// Set the exit gateway node
    SetExit {
        #[command(flatten)]
        exit: CliExit,
    },

    /// Enable or disable IPv6 in the tunnel
    SetIpv6 {
        /// Set IPv6 support state (on|off)
        #[arg(value_parser = BooleanOption::value_parser(), value_name = "on|off")]
        enabled: BooleanOption,
    },

    /// Enable or disable two-hop mode
    SetTwoHop {
        /// Set two-hop mode (on|off)
        #[arg(value_parser = BooleanOption::value_parser(), value_name = "on|off")]
        enabled: BooleanOption,
    },

    /// Enable or disable netstack based implementation for WireGuard
    SetNetstack {
        /// Set netstack implementation (on|off)
        #[arg(value_parser = BooleanOption::value_parser(), value_name = "on|off")]
        enabled: BooleanOption,
    },

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
    pub entry: LegacyCliEntry,

    #[command(flatten)]
    pub exit: LegacyCliExit,

    /// Set the IP address of the DNS server to use.
    #[arg(long)]
    pub dns: Option<IpAddr>,

    /// Disable IPv6 support
    #[arg(long)]
    pub disable_ipv6: bool,

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
}

impl ConnectArgs {
    pub fn entry_point(&self) -> Result<Option<EntryPoint>> {
        self.entry.entry_point()
    }

    pub fn exit_point(&self) -> Result<Option<ExitPoint>> {
        self.exit.exit_point()
    }
}

#[derive(Args)]
#[group(multiple = false, required = true)]
pub struct CliEntry {
    /// Mixnet public ID of the entry gateway.
    #[arg(long)]
    pub id: Option<String>,

    /// Auto-select entry gateway by country ISO.
    #[arg(long)]
    pub country: Option<celes::Country>,

    /// Auto-select entry gateway randomly.
    #[arg(long)]
    pub random: bool,
}

impl CliEntry {
    pub fn entry_point(&self) -> Result<EntryPoint> {
        if let Some(ref entry_gateway_id) = self.id {
            Ok(EntryPoint::Gateway {
                identity: NodeIdentity::from_base58_string(entry_gateway_id)
                    .map_err(|_| anyhow!("Failed to parse gateway id"))?,
            })
        } else if let Some(ref entry_gateway_country) = self.country {
            Ok(EntryPoint::Country {
                two_letter_iso_country_code: entry_gateway_country.alpha2.to_string(),
            })
        } else if self.random {
            Ok(EntryPoint::Random)
        } else {
            unreachable!()
        }
    }
}

#[derive(Args)]
#[group(multiple = false, required = true)]
pub struct CliExit {
    /// Mixnet recipient address of the IPR connecting to, if specified directly. This is only
    /// useful when connecting to standalone IPRs.
    #[clap(long, hide = true)]
    pub ipr_address: Option<String>,

    /// Mixnet public ID of the exit gateway.
    #[clap(long)]
    pub id: Option<String>,

    /// Auto-select exit gateway by country ISO.
    #[clap(long)]
    pub country: Option<celes::Country>,

    /// Auto-select exit gateway by region.
    #[clap(long)]
    pub region: Option<String>,

    /// Auto-select exit gateway randomly.
    #[clap(long)]
    pub random: bool,
}

impl CliExit {
    pub fn exit_point(&self) -> Result<ExitPoint> {
        if let Some(ref exit_router_address) = self.ipr_address {
            Ok(ExitPoint::Address {
                address: Box::new(
                    Recipient::try_from_base58_string(exit_router_address)
                        .map_err(|_| anyhow!("Failed to parse exit node address"))?,
                ),
            })
        } else if let Some(ref exit_router_id) = self.id {
            Ok(ExitPoint::Gateway {
                identity: NodeIdentity::from_base58_string(exit_router_id.clone())
                    .map_err(|_| anyhow!("Failed to parse gateway id"))?,
            })
        } else if let Some(ref exit_gateway_country) = self.country {
            Ok(ExitPoint::Country {
                two_letter_iso_country_code: exit_gateway_country.alpha2.to_string(),
            })
        } else if let Some(ref exit_gateway_region) = self.region {
            Ok(ExitPoint::Region {
                region: exit_gateway_region.to_string(),
            })
        } else if self.random {
            Ok(ExitPoint::Random)
        } else {
            unreachable!()
        }
    }
}

impl TryFrom<CliExit> for ExitPoint {
    type Error = anyhow::Error;

    fn try_from(value: CliExit) -> std::result::Result<Self, Self::Error> {
        if let Some(ref exit_router_address) = value.ipr_address {
            Ok(ExitPoint::Address {
                address: Box::new(
                    Recipient::try_from_base58_string(exit_router_address)
                        .map_err(|_| anyhow!("Failed to parse exit node address"))?,
                ),
            })
        } else if let Some(ref exit_router_id) = value.id {
            Ok(ExitPoint::Gateway {
                identity: NodeIdentity::from_base58_string(exit_router_id.clone())
                    .map_err(|_| anyhow!("Failed to parse gateway id"))?,
            })
        } else if let Some(ref exit_gateway_country) = value.country {
            Ok(ExitPoint::Country {
                two_letter_iso_country_code: exit_gateway_country.alpha2.to_string(),
            })
        } else if let Some(ref exit_gateway_region) = value.region {
            Ok(ExitPoint::Region {
                region: exit_gateway_region.to_string(),
            })
        } else if value.random {
            Ok(ExitPoint::Random)
        } else {
            Err(anyhow!("Invalid Exit Point value"))
        }
    }
}

#[derive(Args)]
#[group(multiple = false)]
pub struct LegacyCliEntry {
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

impl LegacyCliEntry {
    pub fn entry_point(&self) -> Result<Option<EntryPoint>> {
        if let Some(ref entry_gateway_id) = self.entry_id {
            Ok(Some(EntryPoint::Gateway {
                identity: NodeIdentity::from_base58_string(entry_gateway_id)
                    .map_err(|_| anyhow!("Failed to parse gateway id"))?,
            }))
        } else if let Some(ref entry_gateway_country) = self.entry_country {
            Ok(Some(EntryPoint::Country {
                two_letter_iso_country_code: entry_gateway_country.alpha2.to_string(),
            }))
        } else if self.entry_random {
            Ok(Some(EntryPoint::Random))
        } else {
            Ok(None)
        }
    }
}

#[derive(Args)]
#[group(multiple = false)]
pub struct LegacyCliExit {
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

    /// Auto-select exit gateway by region.
    #[clap(long, alias = "exit-gateway-region")]
    pub exit_region: Option<String>,

    /// Auto-select exit gateway randomly.
    #[clap(long, alias = "exit-gateway-random")]
    pub exit_random: bool,
}

impl LegacyCliExit {
    pub fn exit_point(&self) -> Result<Option<ExitPoint>> {
        if let Some(ref exit_router_address) = self.exit_ipr_address {
            Ok(Some(ExitPoint::Address {
                address: Box::new(
                    Recipient::try_from_base58_string(exit_router_address)
                        .map_err(|_| anyhow!("Failed to parse exit node address"))?,
                ),
            }))
        } else if let Some(ref exit_router_id) = self.exit_id {
            Ok(Some(ExitPoint::Gateway {
                identity: NodeIdentity::from_base58_string(exit_router_id.clone())
                    .map_err(|_| anyhow!("Failed to parse gateway id"))?,
            }))
        } else if let Some(ref exit_gateway_country) = self.exit_country {
            Ok(Some(ExitPoint::Country {
                two_letter_iso_country_code: exit_gateway_country.alpha2.to_string(),
            }))
        } else if let Some(ref exit_gateway_region) = self.exit_region {
            Ok(Some(ExitPoint::Region {
                region: exit_gateway_region.to_string(),
            }))
        } else if self.exit_random {
            Ok(Some(ExitPoint::Random))
        } else {
            Ok(None)
        }
    }
}

impl TryFrom<LegacyCliExit> for ExitPoint {
    type Error = anyhow::Error;

    fn try_from(value: LegacyCliExit) -> std::result::Result<Self, Self::Error> {
        if let Some(ref exit_router_address) = value.exit_ipr_address {
            Ok(ExitPoint::Address {
                address: Box::new(
                    Recipient::try_from_base58_string(exit_router_address)
                        .map_err(|_| anyhow!("Failed to parse exit node address"))?,
                ),
            })
        } else if let Some(ref exit_router_id) = value.exit_id {
            Ok(ExitPoint::Gateway {
                identity: NodeIdentity::from_base58_string(exit_router_id.clone())
                    .map_err(|_| anyhow!("Failed to parse gateway id"))?,
            })
        } else if let Some(ref exit_gateway_country) = value.exit_country {
            Ok(ExitPoint::Country {
                two_letter_iso_country_code: exit_gateway_country.alpha2.to_string(),
            })
        } else if let Some(ref exit_gateway_region) = value.exit_region {
            Ok(ExitPoint::Region {
                region: exit_gateway_region.to_string(),
            })
        } else if value.exit_random {
            Ok(ExitPoint::Random)
        } else {
            Err(anyhow!("Invalid Exit Point value"))
        }
    }
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

/// A value parser that parses "on" or "off" into a boolean
#[derive(Debug, Clone, Copy)]
pub struct BooleanOption {
    state: bool,
    on_label: &'static str,
    off_label: &'static str,
}

impl Deref for BooleanOption {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl clap::builder::ValueParserFactory for BooleanOption {
    type Parser = ValueParser;

    /// A value parser that parses "on" or "off" into a `BooleanOption`
    fn value_parser() -> Self::Parser {
        Self::custom_parser("on", "off")
    }
}

impl BooleanOption {
    /// A value parser that parses `on_label` and `off_label` into a `BooleanOption`
    fn custom_parser(on_label: &'static str, off_label: &'static str) -> ValueParser {
        assert!(on_label != off_label);

        ValueParser::new(
            PossibleValuesParser::new([on_label, off_label])
                .map(move |val| Self::with_labels(val == on_label, on_label, off_label)),
        )
    }

    fn with_labels(state: bool, on_label: &'static str, off_label: &'static str) -> Self {
        Self {
            state,
            on_label,
            off_label,
        }
    }
}

impl From<bool> for BooleanOption {
    fn from(state: bool) -> Self {
        Self::with_labels(state, "on", "off")
    }
}

impl std::fmt::Display for BooleanOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.state {
            self.on_label.fmt(f)
        } else {
            self.off_label.fmt(f)
        }
    }
}
