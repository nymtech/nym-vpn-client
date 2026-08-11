// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod boolean_option;
mod commands;
mod display_helpers;
mod fs;
mod table_style;

use anyhow::{Context, Result, bail};
use clap::{ArgAction, Parser};
use tokio_stream::StreamExt;

use nym_vpn_lib_types::{TunnelEvent, TunnelState, VpnServiceInfo};
use nym_vpn_proto::rpc_client::RpcClient;

use crate::table_style::TableStyle;

#[tokio::main]
async fn main() -> Result<()> {
    let args = ProgramArgs::parse();

    if args.command.needs_rpc_client() {
        let rpc_client = RpcClient::new()
            .await
            .context("Failed to create RPC client")?;

        args.command.execute(rpc_client).await
    } else {
        args.command.execute_no_rpc_client().await
    }
}

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct ProgramArgs {
    /// Table style output.
    #[arg(global = true, long, value_enum, default_value_t)]
    pub table_style: TableStyle,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Create an interactive session to issue commands with a one time, initial authentication
    StartSession,

    /// Exit a started interactive session
    #[clap(alias = "exit")]
    ExitSession,

    /// Connect the tunnel
    #[clap(alias = "connect-v2")]
    Connect {
        /// Blocks until the connection is established or failed
        #[arg(short, long)]
        wait: bool,
    },

    /// Reconnect the tunnel to any matching gateway
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
        #[arg(short, long, default_value = "false", action = ArgAction::SetTrue)]
        listen: bool,
    },

    /// Get info about the current client. Things like version and network details.
    Info,

    /// Get the current VPN service configuration.
    #[clap(hide = true)]
    GetConfig,

    /// Manage entry and exit gateway nodes, list available gateways
    Gateway(commands::gateway::Args),

    /// Local network policy
    Lan {
        #[command(subcommand)]
        subcommand: commands::lan::Command,
    },

    /// Ad blocking
    #[clap(alias = "adblock")]
    AdBlock {
        #[command(subcommand)]
        subcommand: commands::ad_block::Command,
    },

    /// DNS
    Dns {
        #[command(subcommand)]
        subcommand: commands::dns::Command,
    },

    /// Tunnel configuration (enable or disable ipv6, two-hop mode, circumvention transports)
    Tunnel {
        #[command(subcommand)]
        subcommand: commands::tunnel::Command,
    },

    /// Account information
    Account {
        #[command(subcommand)]
        subcommand: commands::account::Command,
    },

    /// Device information
    Device(commands::device::Args),

    /// Nym network configuration
    Network {
        #[command(subcommand)]
        subcommand: commands::network::Command,
    },

    /// Sentry integration
    Sentry {
        #[command(subcommand)]
        subcommand: commands::sentry::Command,
    },

    /// SOCKS5 proxy
    Socks5 {
        #[command(subcommand)]
        subcommand: commands::socks5::Command,
    },

    /// Geo exclusion configuration
    GeoExclusion {
        #[command(subcommand)]
        subcommand: commands::geo_exclusion::Command,
    },

    /// Anonymous network statistics collection
    NetworkStats {
        #[command(subcommand)]
        subcommand: commands::network_stats::Command,
    },

    /// Diagnostic tool
    Diagnostic {
        #[command(subcommand)]
        subcommand: nym_diagnostic::cli::Command,
    },

    /// Favorites management
    Favorites {
        #[command(subcommand)]
        subcommand: commands::favorites::Command,
    },

    /// Split tunneling
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    SplitTunnel {
        #[command(subcommand)]
        subcommand: commands::split_tunnel::Command,
    },

    /// Profiles management.
    Profile {
        #[command(subcommand)]
        subcommand: commands::profile::Command,
    },
}

impl Command {
    pub fn needs_rpc_client(&self) -> bool {
        !matches!(self, Self::Favorites { .. })
    }

    pub async fn execute_no_rpc_client(self) -> Result<()> {
        match self {
            Command::Favorites { subcommand } => subcommand.execute().await,
            _ => Err(anyhow::anyhow!("internal: command needs rpc client")),
        }
    }

    pub async fn execute(self, rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::StartSession => commands::session::execute(rpc_client).await,
            Command::ExitSession => Ok(()),
            Command::Connect { wait } => Self::connect(rpc_client, wait).await,
            Command::Reconnect => Self::reconnect(rpc_client).await,
            Command::Disconnect { wait } => Self::disconnect(rpc_client, wait).await,
            Command::Status { listen } => Self::status(rpc_client, listen).await,
            Command::Info => Self::info(rpc_client).await,
            Command::GetConfig => Self::get_config(rpc_client).await,
            Command::Gateway(args) => args.execute(rpc_client).await,
            Command::Tunnel { subcommand } => subcommand.execute(rpc_client).await,
            Command::Lan { subcommand } => subcommand.execute(rpc_client).await,
            Command::AdBlock { subcommand } => subcommand.execute(rpc_client).await,
            Command::Dns { subcommand } => subcommand.execute(rpc_client).await,
            Command::Network { subcommand } => subcommand.execute(rpc_client).await,
            Command::Account { subcommand } => subcommand.execute(rpc_client).await,
            Command::Device(args) => args.execute(rpc_client).await,
            Command::Sentry { subcommand } => subcommand.execute(rpc_client).await,
            Command::Socks5 { subcommand } => subcommand.execute(rpc_client).await,
            Command::GeoExclusion { subcommand } => subcommand.execute(rpc_client).await,
            Command::NetworkStats { subcommand } => subcommand.execute(rpc_client).await,
            Command::Diagnostic { subcommand } => {
                commands::diagnostic::execute(subcommand, rpc_client).await
            }
            Command::Favorites { subcommand } => subcommand.execute().await,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Command::SplitTunnel { subcommand } => subcommand.execute(rpc_client).await,
            Command::Profile { subcommand } => subcommand.execute(rpc_client).await,
        }
    }

    async fn connect(mut rpc_client: RpcClient, wait: bool) -> Result<()> {
        rpc_client.connect_tunnel().await?;

        if wait {
            println!("Waiting until connected or failed");
            Self::wait_until_connected(rpc_client).await
        } else {
            Ok(())
        }
    }

    async fn reconnect(mut rpc_client: RpcClient) -> Result<()> {
        let _accepted = rpc_client.reconnect_tunnel().await?;
        Ok(())
    }

    async fn wait_until_connected(mut rpc_client: RpcClient) -> Result<()> {
        let mut stream = rpc_client.listen_to_events().await?;
        while let Some(new_state) = stream.next().await {
            let TunnelEvent::NewState(new_state) = new_state? else {
                continue;
            };
            println!("{new_state}");

            match new_state {
                TunnelState::Connected { .. } => {
                    break;
                }
                TunnelState::Offline { reconnect } => {
                    if reconnect {
                        println!("Device is offline. Waiting for network connectivity.");
                    } else {
                        bail!("Device is offline");
                    }
                }
                TunnelState::Error(reason) => {
                    bail!("Tunnel entered error state {reason:?}");
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn disconnect(mut rpc_client: RpcClient, wait: bool) -> Result<()> {
        if wait {
            let mut stream = rpc_client.clone().listen_to_events().await?;

            println!("Waiting until disconnected");

            let current_state = rpc_client.get_tunnel_state().await?;
            println!("{current_state}");

            rpc_client.disconnect_tunnel().await?;

            if matches!(
                current_state,
                TunnelState::Disconnected | TunnelState::Offline { .. }
            ) {
                return Ok(());
            }

            while let Some(new_state) = stream.next().await {
                let TunnelEvent::NewState(new_state) = new_state? else {
                    continue;
                };

                println!("{new_state}");

                if matches!(
                    new_state,
                    TunnelState::Disconnected | TunnelState::Offline { .. }
                ) {
                    return Ok(());
                } else if let TunnelState::Error(reason) = new_state {
                    bail!("Tunnel entered error state: {reason:?}")
                }
            }
            Ok(())
        } else {
            rpc_client.disconnect_tunnel().await?;
            Ok(())
        }
    }

    async fn status(mut rpc_client: RpcClient, listen: bool) -> Result<()> {
        let state = rpc_client.get_tunnel_state().await?;
        println!("State: {state}");

        if !listen {
            return Ok(());
        }

        let mut stream = rpc_client.listen_to_events().await?;

        while let Some(event) = stream.next().await {
            match event {
                Ok(TunnelEvent::NewState(new_state)) => {
                    println!("State: {new_state}");
                }
                Ok(TunnelEvent::ConfigChanged(new_config)) => {
                    let json = serde_json::to_string_pretty(&new_config)
                        .context("failed to convert new config to JSON")?;
                    println!("Configuration changed:\n{json}");
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn info(mut rpc_client: RpcClient) -> Result<()> {
        let service_info = rpc_client.get_info().await?;
        Self::print_service_info(service_info);
        Ok(())
    }

    async fn get_config(mut rpc_client: RpcClient) -> Result<()> {
        let config = rpc_client.get_config().await?;
        println!("{config:#?}");
        Ok(())
    }

    fn print_service_info(service_info: VpnServiceInfo) {
        println!("nym-vpnd:");
        println!("  version: {}", service_info.version);
        println!(
            "  build_timestamp (utc): {}",
            service_info
                .build_timestamp
                .map(|s| s.to_string())
                .unwrap_or_default()
        );
        println!("  triple: {}", service_info.triple);
        println!("  platform: {}", service_info.platform);
        println!("  git_commit: {}", service_info.git_commit);
        println!();
        println!("nym_network:");
        println!("  network_name: {}", service_info.nym_network.network_name);
        println!("  chain_details:");
        println!(
            "    bech32_account_prefix: {}",
            service_info.nym_network.chain_details.bech32_account_prefix
        );
        println!("    mix_denom:");
        println!(
            "      base: {}",
            service_info.nym_network.chain_details.mix_denom.base
        );
        println!(
            "      display: {}",
            service_info.nym_network.chain_details.mix_denom.display
        );
        println!(
            "      display_exponent: {}",
            service_info
                .nym_network
                .chain_details
                .mix_denom
                .display_exponent
        );
        println!("    stake_denom:");
        println!(
            "      base: {}",
            service_info.nym_network.chain_details.stake_denom.base
        );
        println!(
            "      display: {}",
            service_info.nym_network.chain_details.stake_denom.display
        );
        println!(
            "      display_exponent: {}",
            service_info
                .nym_network
                .chain_details
                .stake_denom
                .display_exponent
        );

        println!("  validators:");
        for validator in &service_info.nym_network.endpoints {
            println!("    nyxd_url: {}", validator.nyxd_url);
            println!("    api_url: {}", or_not_set(&validator.api_url));
            println!(
                "    websocket_url: {}",
                or_not_set(&validator.websocket_url)
            );
        }

        println!("  nym_contracts:");
        println!(
            "    mixnet_contract_address: {}",
            or_not_set(&service_info.nym_network.contracts.mixnet_contract_address)
        );
        println!(
            "    vesting_contract_address: {}",
            or_not_set(&service_info.nym_network.contracts.vesting_contract_address)
        );
        println!(
            "    ecash_contract_address: {}",
            or_not_set(&service_info.nym_network.contracts.ecash_contract_address)
        );
        println!(
            "    group_contract_address: {}",
            or_not_set(&service_info.nym_network.contracts.group_contract_address)
        );
        println!(
            "    multisig_contract_address: {}",
            or_not_set(&service_info.nym_network.contracts.multisig_contract_address)
        );
        println!(
            "    coconut_dkg_contract_address: {}",
            or_not_set(
                &service_info
                    .nym_network
                    .contracts
                    .coconut_dkg_contract_address
            )
        );
        if let Some(nym_api_urls) = service_info.nym_network.nym_api_urls {
            println!("  nym_api_urls:");
            for nym_api_url in nym_api_urls {
                println!("    - url: {}", nym_api_url.url);
                match nym_api_url.front_hosts {
                    Some(fronts) => {
                        println!(
                            "      fronts: {}",
                            fronts.into_iter().collect::<Vec<String>>().join(", ")
                        );
                    }
                    None => println!("      fronts: (unset)"),
                }
            }
        }
        println!("  nym_vpn_api_urls:");
        if let Some(nym_vpn_api_urls) = service_info.nym_network.nym_vpn_api_urls {
            for nym_vpn_api_url in nym_vpn_api_urls {
                println!("    - url: {}", nym_vpn_api_url.url);
                match nym_vpn_api_url.front_hosts {
                    Some(fronts) => {
                        println!(
                            "      fronts: {}",
                            fronts.into_iter().collect::<Vec<String>>().join(", ")
                        );
                    }
                    None => println!("      fronts: (unset)"),
                }
            }
        }

        println!();
        println!("nym_vpn_network:");
        println!("  nym_vpn_api_urls:");
        for nym_vpn_api_url in service_info.nym_vpn_network.nym_vpn_api_urls {
            println!("    - url: {}", nym_vpn_api_url.url);
            match nym_vpn_api_url.front_hosts {
                Some(fronts) => {
                    println!(
                        "      fronts: {}",
                        fronts.into_iter().collect::<Vec<String>>().join(", ")
                    );
                }
                None => println!("      fronts: (unset)"),
            }
        }
    }
}

pub fn or_not_set<T: ToString + Clone>(value: &Option<T>) -> String {
    value
        .clone()
        .map(|v| v.to_string())
        .unwrap_or("not set".to_string())
}
