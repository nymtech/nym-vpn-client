// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use nym_vpn_lib_types::StoreAccountRequest;
use nym_vpn_proto::rpc_client::RpcClient;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get current account information
    Get,
    /// Login with mnemonic
    Set {
        /// Mnemonic phrase or private key hex encoded
        #[arg(index = 1)]
        secret: String,
        /// Type of login of the secret
        #[clap(long, default_value_t = LoginType::Mnemonic)]
        login_type: LoginType,
        /// Account mode
        #[clap(long, default_value_t = VpnAccountMode::Api)]
        mode: VpnAccountMode,
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
    /// Attempt to obtain additional accounts for a 'decentralised' account
    DecentralisedObtainTicketbooks {
        /// Amount of ticketbooks (per type) to attempt to obtain
        #[arg(long, default_value_t = 1)]
        amount: u64,
    },
    /// Refresh account state
    #[clap(hide = true)]
    Refresh,
    /// Get available tickets
    #[clap(hide = true)]
    AvailableTickets,
    /// Get account usage
    #[clap(hide = true)]
    Usage,
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Get => {
                let account_id = rpc_client.get_account_identity().await?;
                let account_state = rpc_client.get_account_state().await?;

                println!(
                    "Account identity: {}",
                    account_id.unwrap_or("unset".to_owned())
                );
                println!("Account state: {account_state:?}");
                Ok(())
            }
            Command::Set {
                secret,
                login_type,
                mode,
            } => {
                let secret = match login_type {
                    LoginType::Mnemonic => nym_vpn_lib_types::LoginSecret::Mnemonic(secret),
                    LoginType::PrivateKeyHex => {
                        nym_vpn_lib_types::LoginSecret::PrivyHexSignature(secret)
                    }
                };
                let request = match mode {
                    VpnAccountMode::Api => StoreAccountRequest::Vpn { secret },
                    VpnAccountMode::Decentralised => StoreAccountRequest::Decentralised { secret },
                };
                let response = rpc_client.store_account(request).await?;

                if let Some(err) = response.error {
                    println!("Failed to set account: {err}");
                    return Err(err.into());
                } else {
                    println!("Your account has been set. Welcome to the Nym VPN!");
                }

                Ok(())
            }
            Command::Forget => {
                let response = rpc_client.forget_account().await?;
                if let Some(err) = response.error {
                    println!("Failed to forget account: {err}");
                    return Err(err.into());
                } else {
                    println!("Account forgotten successfully");
                }
                Ok(())
            }
            Command::RotateKeys => {
                let response = rpc_client.rotate_keys().await?;
                if let Some(err) = response.error {
                    println!("Failed to rotate keys: {err}");
                    return Err(err.into());
                } else {
                    println!("Keys rotated successfully");
                }
                Ok(())
            }
            Command::Links { locale } => {
                let account_links = rpc_client.get_account_links(locale).await?;

                println!("Sign up: {}", account_links.sign_up);
                println!("Sign in: {}", account_links.sign_in);
                if let Some(account_url) = account_links.account {
                    println!("Account: {account_url}");
                }

                Ok(())
            }
            Command::Balance => {
                let response = rpc_client.account_balance().await?;
                match response.result {
                    Err(err) => {
                        println!("Failed to get account balance: {err}");
                        return Err(err.into());
                    }
                    Ok(balance) => println!(
                        "account balance: {}",
                        balance
                            .into_iter()
                            .map(|c| c.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    ),
                }
                Ok(())
            }
            Command::DecentralisedObtainTicketbooks { amount } => {
                println!(
                    "starting acquisition of {amount} ticketbooks (per type). this might take a while..."
                );
                let response = rpc_client.decentralised_obtain_ticketbooks(amount).await?;
                if let Some(err) = response.error {
                    println!("Failed to obtain ticketbooks: {err}");
                    return Err(err.into());
                } else {
                    println!("Successfully managed to obtain {amount} (per type) ticketbooks!");
                }

                Ok(())
            }
            Command::Refresh => {
                rpc_client.refresh_account_state().await?;
                Ok(())
            }
            Command::AvailableTickets => {
                let response = rpc_client.get_available_tickets().await?;
                println!("{response:#?}");
                Ok(())
            }
            Command::Usage => {
                let response = rpc_client.get_account_usage().await?;
                println!("{response:#?}");
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum VpnAccountMode {
    Api,
    Decentralised,
}

impl std::fmt::Display for VpnAccountMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VpnAccountMode::Api => write!(f, "api"),
            VpnAccountMode::Decentralised => write!(f, "decentralised"),
        }
    }
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum LoginType {
    Mnemonic,
    PrivateKeyHex,
}

impl std::fmt::Display for LoginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginType::Mnemonic => write!(f, "mnemonic"),
            LoginType::PrivateKeyHex => write!(f, "private key"),
        }
    }
}
