// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, anyhow};
use nym_vpn_lib_types::{ExitPoint, HttpRpcSettings, NodeIdentity, Recipient, Socks5Settings};
use nym_vpn_proto::rpc_client::RpcClient;
use std::net::SocketAddr;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Enable SOCKS5 proxy
    ///
    /// For example:
    ///
    /// nym-vpnc socks5 enable --socks5-address=127.0.0.1:1080 --rpc-address=127.0.0.1:8545 --exit-country=GB
    Enable(Box<EnableArgs>),

    /// Disable Custom DNS
    Disable,

    /// Get SOCKS5 proxy status
    Status,
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Enable(args) => {
                // If the SOCKS5 server is currently enabled then any new settings will
                // be ignored, so block that here
                let status = rpc_client.get_socks5_status().await?;
                if status.state != nym_vpn_lib_types::Socks5State::Disabled {
                    return Err(anyhow!(
                        "SOCKS5 proxy is already enabled; disable it before changing settings"
                    ));
                }

                let socks5_settings = Socks5Settings {
                    listen_address: Some(args.socks5_address),
                };
                let http_rpc_settings = HttpRpcSettings {
                    listen_address: args.rpc_address,
                };
                let exit_point = args.exit_point()?;
                rpc_client
                    .enable_socks5(socks5_settings, http_rpc_settings, exit_point)
                    .await?;
                Ok(())
            }
            Command::Disable => {
                rpc_client.disable_socks5().await?;
                Ok(())
            }
            Command::Status => {
                let status = rpc_client.get_socks5_status().await?;
                println!("State: {:?}", status.state);
                println!(
                    "SOCKS5 Listen Address: {:?}",
                    status.socks5_settings.listen_address
                );
                println!(
                    "HTTP RPC Listen Address: {:?}",
                    status.http_rpc_settings.listen_address
                );
                println!(
                    "Error: {}",
                    status.error_message.unwrap_or_else(|| "None".to_string())
                );
                println!("Active Connections: {}", status.active_connections);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, clap::Args)]
#[command(group = clap::ArgGroup::new("exit").multiple(false))]
pub struct EnableArgs {
    /// SOCKS5 listen address
    #[arg(long)]
    pub socks5_address: SocketAddr,

    /// HTTPS proxy address
    #[arg(long)]
    pub rpc_address: Option<SocketAddr>,

    /// Mixnet recipient address of the IPR connecting to, if specified directly. This is only
    /// useful when connecting to standalone IPRs.
    #[arg(long, group = "exit", hide = true)]
    pub exit_ipr_address: Option<String>,

    /// Mixnet public ID of the exit gateway.
    #[arg(long, group = "exit")]
    pub exit_id: Option<String>,

    /// Auto-select exit gateway by country ISO.
    #[arg(long, group = "exit")]
    pub exit_country: Option<celes::Country>,

    /// Auto-select exit gateway by region.
    #[arg(long, group = "exit")]
    pub exit_region: Option<String>,

    /// Auto-select exit gateway randomly.
    #[arg(long, action = clap::ArgAction::SetTrue, group = "exit")]
    pub exit_random: bool,
}

impl EnableArgs {
    pub fn exit_point(&self) -> Result<ExitPoint> {
        if let Some(ref exit_router_address) = self.exit_ipr_address {
            Ok(ExitPoint::Address {
                address: Box::new(
                    Recipient::try_from_base58_string(exit_router_address)
                        .map_err(|_| anyhow!("Failed to parse exit node address"))?,
                ),
            })
        } else if let Some(ref exit_router_id) = self.exit_id {
            Ok(ExitPoint::Gateway {
                identity: NodeIdentity::from_base58_string(exit_router_id.clone())
                    .map_err(|_| anyhow!("Failed to parse gateway id"))?,
            })
        } else if let Some(ref exit_gateway_country) = self.exit_country {
            Ok(ExitPoint::Country {
                two_letter_iso_country_code: exit_gateway_country.alpha2.to_string(),
            })
        } else if let Some(ref exit_gateway_region) = self.exit_region {
            Ok(ExitPoint::Region {
                region: exit_gateway_region.to_string(),
            })
        } else if self.exit_random {
            Ok(ExitPoint::Random)
        } else {
            Err(anyhow!("No exit point specified"))
        }
    }
}
