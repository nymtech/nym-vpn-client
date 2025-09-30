// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use anyhow::{Result, anyhow};

use nym_vpn_lib_types::{
    ConnectArgs as DaemonConnectArgs, ConnectOptions, EntryPoint, ExitPoint, NodeIdentity,
    Recipient, StoreAccountRequest, UserAgent,
};
use nym_vpn_proto::rpc_client::RpcClient;

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Connect to the Nym network (deprecated, use instead: nym-vpnc connect-v2)
    /// Individual tunnel parameters are configured separately. Learn more by running:
    /// - nym-vpnc tunnel --help
    /// - nym-vpnc gateway --help
    #[clap(verbatim_doc_comment)]
    Connect(Box<ConnectArgs>),

    /// Set the network to be used. This requires a restart of the daemon (`nym-vpnd`) (deprecated, use instead: nym-vpnc network set <network>)
    SetNetwork(SetNetworkArgs),

    /// Store the account recovery phrase. (deprecated, use instead: nym-vpnc account set <mnemonic>)
    StoreAccount(StoreAccountArgs),

    /// Check if the account is stored. (deprecated, use instead: nym-vpnc account get)
    IsAccountStored,

    /// Forget the stored account. This removes the stores recovery phrase, device and mixnet keys,
    /// stored local credentials, etc. (deprecated, use instead: nym-vpnc account forget)
    ForgetAccount,

    /// Get the account ID. (deprecated, use instead: nym-vpnc account get)
    GetAccountId,

    /// Get current account state. (deprecated, use instead: nym-vpnc account get)
    GetAccountState,

    /// Get URLs for managing your nym-vpn account. (deprecated, use instead: nym-vpnc account links --locale <locale>)
    GetAccountLinks(GetAccountLinksArgs),

    /// Get the device ID. (deprecated, use instead: nym-vpnc device get)
    GetDeviceId,

    /// Internal commands for development and debugging. (deprecated)
    #[clap(subcommand, hide = true)]
    Internal(Internal),
}

#[derive(Debug, clap::Subcommand)]
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

impl Command {
    pub async fn execute(self, rpc_client: RpcClient, user_agent: UserAgent) -> Result<()> {
        println!("This call is deprecated and going to be removed soon.");

        match self {
            Command::Connect(connect_args) => connect(rpc_client, *connect_args, user_agent).await,
            Command::SetNetwork(args) => set_network(rpc_client, args).await,
            Command::StoreAccount(store_args) => store_account(rpc_client, store_args).await,
            Command::IsAccountStored => is_account_stored(rpc_client).await,
            Command::ForgetAccount => forget_account(rpc_client).await,
            Command::GetAccountId => get_account_id(rpc_client).await,
            Command::GetAccountLinks(args) => get_account_links(rpc_client, args).await,
            Command::GetAccountState => get_account_state(rpc_client).await,
            Command::GetDeviceId => get_device_id(rpc_client).await,
            Command::Internal(internal) => match internal {
                Internal::GetSystemMessages => get_system_messages(rpc_client).await,
                Internal::GetFeatureFlags => get_feature_flags(rpc_client).await,
                Internal::SyncAccountState => refresh_account_state(rpc_client).await,
                Internal::GetAccountUsage => get_account_usage(rpc_client).await,
                Internal::ResetDeviceIdentity(args) => {
                    reset_device_identity(rpc_client, args).await
                }
                Internal::GetDevices => get_devices(rpc_client).await,
                Internal::GetActiveDevices => get_active_devices(rpc_client).await,
                Internal::GetAvailableTickets => get_available_tickets(rpc_client).await,
            },
        }
    }
}

#[derive(Debug, clap::Args)]
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

    /// Enable two-hop wireguard traffic. This means that traffic jumps directly from entry gateway
    /// to exit gateway using Wireguard protocol.
    #[arg(long)]
    pub enable_two_hop: bool,

    /// Enable Circumvention Transport (CT) wrapping for the connection to the entry gateway in two
    /// hop wireguard mode.
    #[arg(long = "enable-ct", requires = "enable_two_hop")]
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
        self.entry.entry_point()
    }

    pub fn exit_point(&self) -> Result<Option<ExitPoint>> {
        self.exit.exit_point()
    }
}

#[derive(Debug, clap::Args)]
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

#[derive(Debug, clap::Args)]
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

#[derive(Debug, clap::Args)]
pub struct SetNetworkArgs {
    /// The network to be set.
    pub network: String,
}

#[derive(Debug, clap::Args)]
pub struct StoreAccountArgs {
    /// The account mnemonic to be stored.
    #[arg(long)]
    pub mnemonic: String,
}

#[derive(Debug, clap::Args)]
pub struct GetAccountLinksArgs {
    /// The locale to be used.
    #[arg(long)]
    pub locale: String,
}

#[derive(Debug, clap::Args)]
pub struct ResetDeviceIdentityArgs {
    /// Reset the device identity using the given seed.
    #[arg(long)]
    pub seed: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct GetZkNymByIdArgs {
    /// The ID of the ZK Nym to fetch.
    #[arg(short, long)]
    pub id: String,
}

#[derive(Debug, clap::Args)]
pub struct ConfirmZkNymDownloadedArgs {
    /// The ID of the ZK Nym to confirm.
    #[arg(short, long)]
    pub id: String,
}

async fn connect(
    mut rpc_client: RpcClient,
    connect_args: ConnectArgs,
    user_agent: UserAgent,
) -> Result<()> {
    let options = DaemonConnectArgs {
        entry: connect_args.entry_point()?,
        exit: connect_args.exit_point()?,
        options: ConnectOptions {
            dns: connect_args.dns,
            disable_ipv6: connect_args.disable_ipv6,
            enable_two_hop: connect_args.enable_two_hop,
            enable_bridges: connect_args.circumvention_transports,
            netstack: connect_args.netstack,
            disable_poisson_rate: connect_args.disable_poisson_rate,
            disable_background_cover_traffic: connect_args.disable_background_cover_traffic,
            enable_credentials_mode: connect_args.enable_credentials_mode,
            user_agent: Some(user_agent),
        },
    };

    rpc_client.connect_tunnel(options).await?;

    if connect_args.wait {
        println!("Waiting until connected or failed");
        crate::wait_until_connected(rpc_client).await
    } else {
        Ok(())
    }
}

async fn get_device_id(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_device_identity().await?;
    println!("{response:#?}");
    Ok(())
}

async fn get_devices(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_devices().await?;
    println!("{response:#?}");
    Ok(())
}

async fn get_active_devices(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_active_devices().await?;
    println!("{response:#?}");
    Ok(())
}

async fn set_network(mut rpc_client: RpcClient, args: SetNetworkArgs) -> Result<()> {
    rpc_client.set_network(args.network).await?;
    Ok(())
}

async fn get_system_messages(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_system_messages().await?;
    println!("{response:#?}");
    Ok(())
}

async fn get_feature_flags(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_feature_flags().await?;
    println!("{response:#?}");
    Ok(())
}

async fn store_account(mut rpc_client: RpcClient, store_args: StoreAccountArgs) -> Result<()> {
    let request = StoreAccountRequest {
        mnemonic: store_args.mnemonic.clone(),
    };
    let response = rpc_client.store_account(request).await?;

    if let Some(err) = response.error {
        println!("Error: {err}");
    } else {
        println!("Account recovery phrase stored");
    }

    Ok(())
}

async fn refresh_account_state(mut rpc_client: RpcClient) -> Result<()> {
    rpc_client.refresh_account_state().await?;
    Ok(())
}

async fn is_account_stored(mut rpc_client: RpcClient) -> Result<()> {
    let is_stored = rpc_client.is_account_stored().await?;
    if is_stored {
        println!("Account is stored");
    } else {
        println!("No account is stored");
    }
    Ok(())
}

async fn get_account_usage(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_account_usage().await?;
    println!("{response:#?}");
    Ok(())
}

async fn forget_account(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.forget_account().await?;
    if let Some(err) = response.error {
        println!("Error: {err}");
    } else {
        println!("Account forgotten successfully");
    }
    Ok(())
}

async fn get_account_id(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_account_identity().await?;
    println!("{response:#?}");
    Ok(())
}

async fn get_account_links(mut rpc_client: RpcClient, args: GetAccountLinksArgs) -> Result<()> {
    let links = rpc_client.get_account_links(args.locale).await?;
    println!("{links:?}");

    Ok(())
}

async fn get_account_state(mut rpc_client: RpcClient) -> Result<()> {
    let account_state = rpc_client.get_account_state().await?;
    println!("{account_state}");
    Ok(())
}

async fn reset_device_identity(
    mut rpc_client: RpcClient,
    args: ResetDeviceIdentityArgs,
) -> Result<()> {
    let seed = args.seed.map(|seed| seed.into_bytes());
    rpc_client.reset_device_identity(seed).await?;
    Ok(())
}

async fn get_available_tickets(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_available_tickets().await?;
    println!("{response:#?}");
    Ok(())
}
