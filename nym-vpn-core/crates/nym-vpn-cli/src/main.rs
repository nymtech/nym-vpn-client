// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fs, path::PathBuf};

use clap::Parser;
use commands::{CliArgs, ImportCredentialTypeEnum};
use futures::channel::mpsc;
use nym_vpn_lib::{
    gateway_directory::{Config as GatewayConfig, EntryPoint, ExitPoint},
    nym_config::defaults::{setup_env, var_names},
    GenericNymVpnConfig, IpPair, MixnetClientConfig, NodeIdentity, NymVpn, Recipient, SpecificVpn,
};
use time::OffsetDateTime;
use tracing::{debug, error, info};

use crate::{
    commands::Commands,
    error::{Error, Result},
};

mod commands;
mod error;

const CONFIG_DIRECTORY_NAME: &str = "nym-vpn-cli";

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
        Commands::ImportCredential(_) => true,
    };

    if !needs_root {
        debug!("Root privileges not required for this command");
        return Ok(());
    }

    #[cfg(unix)]
    return unix_has_root("nym-vpn-cli");

    #[cfg(windows)]
    return win_has_admin("nym-vpn-cli");

    // Assume we're all good on unknown platforms
    debug!("Platform not supported for root privilege check");
    Ok(())
}

#[cfg(unix)]
pub fn unix_has_root(binary_name: &str) -> Result<()> {
    use tracing::debug;

    if nix::unistd::geteuid().is_root() {
        debug!("Root privileges acquired");
        Ok(())
    } else {
        Err(Error::RootPrivilegesRequired {
            binary_name: binary_name.to_string(),
        })
    }
}

#[cfg(windows)]
pub fn win_has_admin(binary_name: &str) -> Result<()> {
    use tracing::debug;

    if is_elevated::is_elevated() {
        debug!("Admin privileges acquired");
        Ok(())
    } else {
        Err(Error::AdminPrivilegesRequired {
            binary_name: binary_name.to_string(),
        })
    }
}

async fn run() -> Result<()> {
    let args = commands::CliArgs::parse();
    setup_logging(&args);
    debug!("{:?}", nym_bin_common::bin_info_local_vergen!());
    setup_env(args.config_env_file.as_ref());

    check_root_privileges(&args)?;

    let data_path = args.data_path.or(mixnet_data_path());

    match args.command {
        Commands::Run(args) => run_vpn(args, data_path).await,
        Commands::ImportCredential(args) => {
            let data_path = data_path.ok_or(Error::ConfigPathNotSet)?;
            import_credential(args, data_path).await.map(|d| {
                if let Some(d) = d {
                    info!("Credential expiry date: {}", d);
                }
            })
        }
    }
}

async fn run_vpn(args: commands::RunArgs, data_path: Option<PathBuf>) -> Result<()> {
    // Setup gateway directory configuration
    let gateway_config = GatewayConfig::new_from_env(args.min_gateway_performance);
    info!("nym-api: {}", gateway_config.api_url());
    info!(
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
    let generic_config = GenericNymVpnConfig {
        mixnet_client_config: MixnetClientConfig {
            enable_poisson_rate: args.enable_poisson_rate,
            disable_background_cover_traffic: args.disable_background_cover_traffic,
            enable_credentials_mode: args.enable_credentials_mode,
            min_mixnode_performance: args.min_mixnode_performance,
            min_gateway_performance: args.min_gateway_performance,
        },
        data_path,
        gateway_config,
        entry_point: entry_point.clone(),
        exit_point: exit_point.clone(),
        nym_ips,
        nym_mtu: args.nym_mtu,
        dns: args.dns,
        disable_routing: args.disable_routing,
        user_agent: Some(nym_bin_common::bin_info_local_vergen!().into()),
    };

    let nym_vpn: SpecificVpn = if args.wireguard_mode {
        let mut nym_vpn = NymVpn::new_wireguard_vpn(entry_point, exit_point);
        nym_vpn.generic_config = generic_config;
        nym_vpn.into()
    } else {
        let mut nym_vpn = NymVpn::new_mixnet_vpn(entry_point, exit_point);
        nym_vpn.generic_config = generic_config;
        nym_vpn.into()
    };

    let handle = nym_vpn_lib::spawn_nym_vpn(nym_vpn).unwrap();

    register_signal_handler(handle.ctrl_tx());

    handle.wait_until_stopped().await.map_err(Error::VpnLib)
}

#[cfg(unix)]
fn register_signal_handler(vpn_ctrl_tx: mpsc::UnboundedSender<nym_vpn_lib::NymVpnCtrlMessage>) {
    use tokio::signal::unix::{signal, SignalKind};

    tokio::spawn(async move {
        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to setup SIGTERM channel");
        let mut sigquit = signal(SignalKind::quit()).expect("Failed to setup SIGQUIT channel");

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl-C signal");
            }
            _ = sigterm.recv() => {
                info!("Received SIGTERM signal");
            }
            _ = sigquit.recv() => {
                info!("Received SIGQUIT signal");
            }
        }
        info!("Sending stop message to VPN");
        vpn_ctrl_tx
            .unbounded_send(nym_vpn_lib::NymVpnCtrlMessage::Stop)
            .unwrap();
    });
}

#[cfg(not(unix))]
fn register_signal_handler(vpn_ctrl_tx: mpsc::UnboundedSender<nym_vpn_lib::NymVpnCtrlMessage>) {
    tokio::spawn(async move {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl-C signal");
            }
        }
        info!("Sending stop message to VPN");
        vpn_ctrl_tx
            .unbounded_send(nym_vpn_lib::NymVpnCtrlMessage::Stop)
            .unwrap();
    });
}

async fn import_credential(
    args: commands::ImportCredentialArgs,
    data_path: PathBuf,
) -> Result<Option<OffsetDateTime>> {
    info!("Importing credential data into: {}", data_path.display());
    let data: ImportCredentialTypeEnum = args.credential_type.into();
    let raw_credential = match data {
        ImportCredentialTypeEnum::Path(path) => {
            fs::read(path).map_err(Error::FailedToReadCredentialPath)?
        }
        ImportCredentialTypeEnum::Data(data) => parse_encoded_credential_data(&data)?,
    };
    fs::create_dir_all(&data_path).map_err(Error::FailedToCreateCredentialDataPath)?;
    Ok(nym_vpn_lib::credentials::import_credential(raw_credential, data_path).await?)
}

fn parse_encoded_credential_data(raw: &str) -> Result<Vec<u8>> {
    bs58::decode(raw)
        .into_vec()
        .map_err(Error::FailedToParseEncodedCredentialData)
}

fn mixnet_data_path() -> Option<PathBuf> {
    let network_name =
        std::env::var(var_names::NETWORK_NAME).expect("NETWORK_NAME env var not set");
    dirs::data_dir().map(|dir| dir.join(CONFIG_DIRECTORY_NAME).join(network_name))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(err) = run().await {
        error!("Exit with error: {err}");
        eprintln!("An error occurred: {err}");
        std::process::exit(1)
    }
    Ok(())
}
