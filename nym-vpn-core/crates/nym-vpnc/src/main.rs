// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod boolean_option;
mod commands;
mod display_helpers;
mod table_style;

use std::str::FromStr;

use anyhow::{Context, Result, bail};
use clap::{ArgAction, Parser};
use sysinfo::System;
use tokio_stream::StreamExt;

use nym_vpn_lib_types::{TunnelEvent, TunnelState, UserAgent, VpnServiceInfo};
use nym_vpn_proto::rpc_client::RpcClient;

use crate::table_style::TableStyle;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct ProgramArgs {
    /// Table style output.
    #[arg(global = true, long, value_enum, default_value_t)]
    pub table_style: TableStyle,

    /// Override the default user agent string.
    #[arg(long, value_parser = parse_user_agent)]
    pub user_agent: Option<UserAgent>,

    #[command(subcommand)]
    pub command: Command,
}

fn parse_user_agent(s: &str) -> Result<UserAgent, String> {
    UserAgent::from_str(s)
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Connect the tunnel
    ConnectV2 {
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
        #[arg(long, default_value = "false", action = ArgAction::SetTrue)]
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

    /// Anonymous network statistics collection
    NetworkStats {
        #[command(subcommand)]
        subcommand: commands::network_stats::Command,
    },

    #[command(flatten)]
    Legacy(commands::legacy::Command),
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = ProgramArgs::parse();

    let mut rpc_client = RpcClient::new()
        .await
        .context("Failed to create RPC client")?;

    let user_agent = if let Some(user_agent) = args.user_agent {
        user_agent
    } else {
        let daemon_info = rpc_client.get_info().await?;
        construct_user_agent(daemon_info)
    };

    match args.command {
        Command::ConnectV2 { wait } => connect_v2(rpc_client, wait).await,
        Command::Reconnect => reconnect(rpc_client).await,
        Command::Disconnect { wait } => disconnect(rpc_client, wait).await,
        Command::Status { listen } => status(rpc_client, listen).await,
        Command::Info => info(rpc_client).await,
        Command::GetConfig => get_config(rpc_client).await,
        Command::Gateway(args) => args.execute(rpc_client, user_agent).await,
        Command::Tunnel { subcommand } => subcommand.execute(rpc_client).await,
        Command::Lan { subcommand } => subcommand.execute(rpc_client).await,
        Command::Network { subcommand } => subcommand.execute(rpc_client).await,
        Command::Account { subcommand } => subcommand.execute(rpc_client).await,
        Command::Device(args) => args.execute(rpc_client).await,
        Command::Sentry { subcommand } => subcommand.execute(rpc_client).await,
        Command::NetworkStats { subcommand } => subcommand.execute(rpc_client).await,
        Command::Legacy(subcommand) => subcommand.execute(rpc_client, user_agent).await,
    }
}

fn construct_user_agent(daemon_info: VpnServiceInfo) -> UserAgent {
    let bin_info = nym_bin_common::bin_info_local_vergen!();
    let version = format!("{} ({})", bin_info.build_version, daemon_info.version);

    // Construct the platform string similar to how user agents are constructed in web browsers
    let name = System::name().unwrap_or("unknown".to_string());
    let os_long = System::long_os_version().unwrap_or("unknown".to_string());
    let arch = System::cpu_arch();
    let platform = format!("{name}; {os_long}; {arch}");

    let git_commit = format!("{} ({})", bin_info.commit_sha, daemon_info.git_commit);
    UserAgent {
        application: bin_info.binary_name.to_string(),
        version,
        platform,
        git_commit,
    }
}

async fn connect_v2(mut rpc_client: RpcClient, wait: bool) -> Result<()> {
    rpc_client.connect_tunnel_v2().await?;

    if wait {
        println!("Waiting until connected or failed");
        wait_until_connected(rpc_client).await
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
    rpc_client.disconnect_tunnel().await?;

    if wait {
        println!("Waiting until disconnected");
        wait_until_disconnected(rpc_client).await
    } else {
        Ok(())
    }
}

async fn wait_until_disconnected(mut rpc_client: RpcClient) -> Result<()> {
    let mut stream = rpc_client.listen_to_events().await?;

    while let Some(new_state) = stream.next().await {
        let TunnelEvent::NewState(new_state) = new_state? else {
            continue;
        };

        println!("{new_state}");

        match new_state {
            TunnelState::Disconnected | TunnelState::Offline { .. } => {
                break;
            }
            TunnelState::Error(reason) => {
                bail!("Tunnel entered error state: {reason:?}")
            }
            _ => {}
        }
    }
    Ok(())
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
    print_service_info(service_info);
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
    println!();
    println!("nym_vpn_network:");
    println!(
        "  nym_vpn_api_url: {}",
        service_info.nym_vpn_network.nym_vpn_api_url
    )
}

pub fn or_not_set<T: ToString + Clone>(value: &Option<T>) -> String {
    value
        .clone()
        .map(|v| v.to_string())
        .unwrap_or("not set".to_string())
}
