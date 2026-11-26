// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use nym_vpn_lib_types::{DeeplinkClient, DeeplinkKind, GetDeeplinkParams, StoreAccountRequest};
use nym_vpn_proto::rpc_client::RpcClient;

use crate::display_helpers::Rows;

mod pairing;
mod summary;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get current account information
    Get,
    /// Login with mnemonic
    Set {
        /// Mnemonic phrase
        #[arg(index = 1)]
        secret: String,
        /// Account location
        #[clap(long, alias = "mode", default_value_t = AccountLocation::Api)]
        location: AccountLocation,
    },
    /// Forget account
    Forget,
    /// Force wireguard key rotation
    RotateKeys,
    /// Get account links
    Links {
        /// Locale string (i.e en-US)
        #[arg(long)]
        locale: String,
    },
    /// Get account balance
    Balance,
    /// Attempt to obtain additional accounts for a 'blockchain' account
    #[clap(
        name = "obtain-ticketbooks",
        alias = "decentralised-obtain-ticketbooks"
    )]
    ObtainTicketbooks {
        /// Ticketbook source (parsed for future compatibility?, currently ignored)
        #[arg(long, value_enum, default_value_t = TicketbookSource::Smartcontract)]
        source: TicketbookSource,
    },
    /// Manually restock ticketbooks running low, via the bandwidth controller
    #[clap(hide = true)]
    RestockTicketbooks,
    /// Refresh account state
    #[clap(hide = true)]
    Refresh {
        /// Force a fresh fetch from the nym-vpn-api instead of re-evaluating the cached summary
        #[arg(long, short)]
        force: bool,
    },
    /// Get available tickets
    #[clap(hide = true)]
    AvailableTickets,
    /// Get account usage
    #[clap(hide = true)]
    Usage,
    /// Get account summary
    Summary,
    /// Get deeplink
    GetDeeplink {
        /// Deeplink Kind
        #[arg(long)]
        kind: DeeplinkKind,
        /// Deeplink name
        #[arg(long)]
        name: String,
    },
    /// Store deeplink account
    StoreDeeplink {
        /// Deeplink Callback URL (nymvpn://...)
        #[arg(long)]
        url: String,
    },
    /// Pairing subcommand
    Pairing {
        #[command(subcommand)]
        subcommand: pairing::Command,
    },
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Get => {
                let account_id = rpc_client.get_account_identity().await?;
                let canonical_account_id = rpc_client.get_canonical_account_identity().await?;
                let account_mode = rpc_client.get_account_mode().await?;
                let account_state = rpc_client.get_account_state().await?;

                let mut rows = Rows::new();
                rows.row("Identity", account_id.as_deref().unwrap_or("unset"));
                if canonical_account_id != account_id {
                    rows.row(
                        "Canonical identity",
                        canonical_account_id.as_deref().unwrap_or("unset"),
                    );
                }
                rows.row("Mode", summary::display_account_mode(account_mode))
                    .row("State", summary::display_account_state(&account_state))
                    .print();
                Ok(())
            }
            Command::Set { secret, location } => {
                let request = match location {
                    AccountLocation::Api => StoreAccountRequest::Vpn { mnemonic: secret },
                    AccountLocation::Privy => StoreAccountRequest::Privy { mnemonic: secret },
                    AccountLocation::Blockchain => {
                        StoreAccountRequest::Decentralised { mnemonic: secret }
                    }
                };
                let response = rpc_client.store_account(request).await?;

                if let Some(err) = response.error {
                    return Err(err.into());
                }
                println!("Account set");
                Ok(())
            }
            Command::Forget => {
                let response = rpc_client.forget_account().await?;
                if let Some(err) = response.error {
                    return Err(err.into());
                }
                println!("Account forgotten");
                Ok(())
            }
            Command::RotateKeys => {
                let response = rpc_client.rotate_keys().await?;
                if let Some(err) = response.error {
                    return Err(err.into());
                }
                println!("Keys rotated");
                Ok(())
            }
            Command::Links { locale } => {
                let account_links = rpc_client.get_account_links(locale).await?;

                let mut rows = Rows::new();
                rows.row("Sign up", account_links.sign_up)
                    .row("Sign in", account_links.sign_in);
                if let Some(account_url) = account_links.account {
                    rows.row("Account", account_url);
                }
                rows.print();
                Ok(())
            }
            Command::Balance => {
                let response = rpc_client.account_balance().await?;
                let balance = response.result?;
                Rows::new().list("Balance", balance).print();
                Ok(())
            }
            Command::ObtainTicketbooks { source: _ } => {
                // Note: source is currently ignored; backend always uses smartcontract blockchain for tickets.
                println!("Obtaining ticketbooks, this may take a while...");
                let response = rpc_client.decentralised_obtain_ticketbooks().await?;
                if let Some(err) = response.error {
                    return Err(err.into());
                }
                println!("Ticketbooks obtained");
                Ok(())
            }
            Command::RestockTicketbooks => {
                rpc_client.restock_ticketbooks().await?;
                println!("Ticketbook restock scheduled");
                Ok(())
            }
            Command::Refresh { force } => {
                rpc_client.refresh_account_state(force).await?;
                Ok(())
            }
            Command::AvailableTickets => {
                let tickets = rpc_client.get_available_tickets().await?;
                summary::print_available_tickets(&tickets);
                Ok(())
            }
            Command::Usage => {
                let usage = rpc_client.get_account_usage().await?;
                summary::print_usage(&usage);
                Ok(())
            }
            Command::Summary => {
                let response = rpc_client.get_account_summary().await?;
                summary::print_summary(response.as_ref());
                Ok(())
            }
            Command::GetDeeplink { kind, name } => {
                let params = GetDeeplinkParams {
                    client: DeeplinkClient::Desktop,
                    locale: "en".to_string(),
                    kind,
                    name,
                };
                let url = rpc_client.get_deeplink(params).await?;
                println!("Deeplink: {url}");
                Ok(())
            }
            Command::StoreDeeplink { url } => {
                let response = rpc_client.deeplink_store_account(url).await?;
                if let Some(err) = response.error {
                    return Err(err.into());
                }
                println!("Account set");
                Ok(())
            }
            Command::Pairing { subcommand } => subcommand.execute(rpc_client).await,
        }
    }
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum AccountLocation {
    Api,
    Privy,
    #[value(name = "blockchain", aliases = ["decentralised", "decentralized"])]
    Blockchain,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum TicketbookSource {
    #[value(name = "smartcontract")]
    Smartcontract,
}

impl std::fmt::Display for TicketbookSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TicketbookSource::Smartcontract => write!(f, "smartcontract"),
        }
    }
}

impl std::fmt::Display for AccountLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountLocation::Api => write!(f, "api"),
            AccountLocation::Privy => write!(f, "privy"),
            AccountLocation::Blockchain => write!(f, "blockchain"),
        }
    }
}
