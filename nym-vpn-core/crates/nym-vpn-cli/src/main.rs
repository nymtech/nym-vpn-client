// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod commands;
mod error;
mod shutdown_handler;

use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use nym_vpn_api_client::types::GatewayMinPerformance;
use nym_vpn_lib::{
    gateway_directory::{Config as GatewayConfig, EntryPoint, ExitPoint},
    nym_config::defaults::{setup_env, var_names},
    tunnel_state_machine::{
        DnsOptions, GatewayPerformanceOptions, MixnetTunnelOptions, NymConfig, TunnelCommand,
        TunnelEvent, TunnelSettings, TunnelStateMachine, TunnelType,
    },
    IpPair, MixnetClientConfig, NodeIdentity, Recipient,
};
use nym_vpn_store::mnemonic::MnemonicStorage as _;

use commands::{CliArgs, Commands};
use error::{Error, Result};

const CONFIG_DIRECTORY_NAME: &str = "nym-vpn-cli";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let args = commands::CliArgs::parse();
    setup_logging(&args);
    tracing::debug!("{:?}", nym_bin_common::bin_info_local_vergen!());
    setup_env(args.config_env_file.as_ref());

    check_root_privileges(&args)?;

    let data_path = args.data_path.or(mixnet_data_path());

    match args.command {
        Commands::Run(args) => run_vpn(args, data_path).await,
        Commands::StoreAccount(args) => store_account(args, data_path).await,
    }
}

pub(crate) fn setup_logging(args: &CliArgs) {
    let mut filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env()
        .unwrap()
        .add_directive("hyper::proto=info".parse().unwrap())
        .add_directive("netlink_proto=info".parse().unwrap());
    if let Commands::Run(run_args) = &args.command {
        if run_args.wireguard_mode {
            filter = filter
                .add_directive("nym_client_core=warn".parse().unwrap())
                .add_directive("nym_gateway_client=warn".parse().unwrap());
        }
    }

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();
}

fn parse_entry_point(args: &commands::RunArgs) -> Result<EntryPoint> {
    if let Some(ref entry_gateway_id) = args.entry.entry_gateway_id {
        Ok(EntryPoint::Gateway {
            identity: NodeIdentity::from_base58_string(entry_gateway_id.clone())
                .map_err(|_| Error::NodeIdentityFormatting)?,
        })
    } else if let Some(ref entry_gateway_country) = args.entry.entry_gateway_country {
        Ok(EntryPoint::Location {
            location: entry_gateway_country.clone(),
        })
    } else if args.entry.entry_gateway_low_latency {
        Ok(EntryPoint::RandomLowLatency)
    } else {
        Ok(EntryPoint::Random)
    }
}

fn parse_exit_point(args: &commands::RunArgs) -> Result<ExitPoint> {
    if let Some(ref exit_router_address) = args.exit.exit_router_address {
        Ok(ExitPoint::Address {
            address: Recipient::try_from_base58_string(exit_router_address.clone())
                .map_err(|_| Error::RecipientFormatting)?,
        })
    } else if let Some(ref exit_router_id) = args.exit.exit_gateway_id {
        Ok(ExitPoint::Gateway {
            identity: NodeIdentity::from_base58_string(exit_router_id.clone())
                .map_err(|_| Error::NodeIdentityFormatting)?,
        })
    } else if let Some(ref exit_gateway_country) = args.exit.exit_gateway_country {
        Ok(ExitPoint::Location {
            location: exit_gateway_country.clone(),
        })
    } else {
        Ok(ExitPoint::Random)
    }
}

#[allow(unreachable_code)]
fn check_root_privileges(args: &commands::CliArgs) -> Result<()> {
    let needs_root = match &args.command {
        Commands::Run(run_args) => !run_args.disable_routing,
        Commands::StoreAccount(_) => true,
    };

    if !needs_root {
        tracing::debug!("Root privileges not required for this command");
        return Ok(());
    }

    #[cfg(unix)]
    return unix_has_root("nym-vpn-cli");

    #[cfg(windows)]
    return win_has_admin("nym-vpn-cli");

    // Assume we're all good on unknown platforms
    tracing::debug!("Platform not supported for root privilege check");
    Ok(())
}

#[cfg(unix)]
pub(crate) fn unix_has_root(binary_name: &str) -> Result<()> {
    if nix::unistd::geteuid().is_root() {
        tracing::debug!("Root privileges acquired");
        Ok(())
    } else {
        Err(Error::RootPrivilegesRequired {
            binary_name: binary_name.to_string(),
        })
    }
}

#[cfg(windows)]
pub(crate) fn win_has_admin(binary_name: &str) -> Result<()> {
    use tracing::debug;

    if is_elevated::is_elevated() {
        tracing::debug!("Admin privileges acquired");
        Ok(())
    } else {
        Err(Error::AdminPrivilegesRequired {
            binary_name: binary_name.to_string(),
        })
    }
}

async fn run_vpn(args: commands::RunArgs, data_path: Option<PathBuf>) -> anyhow::Result<()> {
    // Setup gateway directory configuration
    let min_gateway_performance = GatewayMinPerformance::from_percentage_values(
        args.min_gateway_mixnet_performance.map(u64::from),
        args.min_gateway_vpn_performance.map(u64::from),
    )
    .map_err(Error::FailedToSetupGatewayPerformanceThresholds)?;

    let gateway_config =
        GatewayConfig::new_from_env().with_min_gateway_performance(min_gateway_performance);

    tracing::info!("nym-api: {}", gateway_config.api_url());
    tracing::info!(
        "nym-vpn-api: {}",
        gateway_config
            .nym_vpn_api_url()
            .map(|url| url.to_string())
            .unwrap_or("unavailable".to_string())
    );

    let entry_point = parse_entry_point(&args)?;
    let exit_point = parse_exit_point(&args)?;
    let nym_ips = if let (Some(ipv4), Some(ipv6)) = (args.nym_ipv4, args.nym_ipv6) {
        Some(IpPair::new(ipv4, ipv6))
    } else {
        None
    };

    let (command_tx, command_rx) = mpsc::unbounded_channel();
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let shutdown_token = CancellationToken::new();

    let dns = args
        .dns
        .map(|ip| DnsOptions::Custom(vec![ip]))
        .unwrap_or_default();

    let tunnel_type = if args.wireguard_mode {
        TunnelType::Wireguard
    } else {
        TunnelType::Mixnet
    };

    let mixnet_client_config = MixnetClientConfig {
        disable_poisson_rate: args.wireguard_mode || args.disable_poisson_rate,
        disable_background_cover_traffic: args.wireguard_mode
            || args.disable_background_cover_traffic,
        min_mixnode_performance: args.min_mixnode_performance,
        min_gateway_performance: args.min_gateway_mixnet_performance,
    };

    let mixnet_tunnel_options = MixnetTunnelOptions {
        interface_addrs: nym_ips,
        mtu: args.nym_mtu,
    };

    let nym_config = NymConfig {
        data_path,
        gateway_config,
    };

    let tunnel_settings = TunnelSettings {
        tunnel_type,
        enable_credentials_mode: args.enable_credentials_mode,
        mixnet_client_config: Some(mixnet_client_config),
        gateway_performance_options: GatewayPerformanceOptions::default(),
        mixnet_tunnel_options,
        entry_point: Box::new(entry_point),
        exit_point: Box::new(exit_point),
        dns,
    };

    let state_machine_handle = TunnelStateMachine::spawn(
        command_rx,
        event_tx,
        nym_config,
        tunnel_settings,
        shutdown_token.child_token(),
    )
    .await
    .with_context(|| "Failed to start a tunnel state machine")?;

    let mut shutdown_join_set = shutdown_handler::install(shutdown_token.clone());

    command_tx
        .send(TunnelCommand::Connect)
        .with_context(|| "Failed to send a connect command.")?;

    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                match event {
                    TunnelEvent::NewState(new_state) => {
                        tracing::info!("New state: {}", new_state);
                    }
                    TunnelEvent::MixnetState(event) => {
                        tracing::info!("Mixnet event: {}", event);
                    }
                }
            }
            _ = shutdown_token.cancelled() => {
                tracing::info!("Cancellation received. Breaking event loop.");
                break;
            }
            else => {
                tracing::info!("Event receiver is closed. Breaking event loop.");
                break;
            }
        }
    }

    tracing::info!("Waiting for state machine to shutdown");
    if let Err(e) = state_machine_handle.await {
        tracing::warn!("Failed to join on state machine handle: {}", e);
    }

    tracing::info!("Aborting signal handlers.");
    shutdown_join_set.shutdown().await;

    tracing::info!("Goodbye.");
    Ok(())
}

async fn store_account(
    args: commands::StoreAccountArgs,
    data_path: Option<PathBuf>,
) -> anyhow::Result<()> {
    let path = data_path.context("Data path not set")?;
    let mnemonic = nym_vpn_store::mnemonic::Mnemonic::parse(&args.mnemonic)
        .context("Failed to parse mnemonic")?;
    let storage = nym_vpn_lib::storage::VpnClientOnDiskStorage::new(path);
    storage
        .store_mnemonic(mnemonic)
        .await
        .context("Failed to store mnemonic")
}

fn mixnet_data_path() -> Option<PathBuf> {
    let network_name =
        std::env::var(var_names::NETWORK_NAME).expect("NETWORK_NAME env var not set");
    dirs::data_dir().map(|dir| dir.join(CONFIG_DIRECTORY_NAME).join(network_name))
}
