// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod commands;

use std::fs;
use std::path::PathBuf;

use commands::ImportCredentialTypeEnum;
use nym_vpn_lib::gateway_directory::{Config as GatewayConfig, EntryPoint, ExitPoint};
use nym_vpn_lib::wg_gateway_client::WgConfig as WgGatewayConfig;
use nym_vpn_lib::{error::*, IpPair, NodeIdentity};
use nym_vpn_lib::{NymVpn, Recipient};

use crate::commands::{override_from_env, wg_override_from_env, Commands};
use clap::Parser;
use log::*;
use nym_vpn_lib::nym_config::defaults::{setup_env, var_names};

const CONFIG_DIRECTORY_NAME: &str = "nym-vpn-cli";

pub fn setup_logging() {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env()
        .unwrap()
        .add_directive("hyper::proto=info".parse().unwrap())
        .add_directive("netlink_proto=info".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();
}

fn parse_entry_point(args: &commands::RunArgs) -> Result<EntryPoint> {
    if let Some(ref entry_gateway_id) = args.entry.entry_gateway_id {
        Ok(EntryPoint::Gateway {
            identity: NodeIdentity::from_base58_string(entry_gateway_id.clone())
                .map_err(|_| Error::NodeIdentityFormattingError)?,
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
                .map_err(|_| Error::RecipientFormattingError)?,
        })
    } else if let Some(ref exit_router_id) = args.exit.exit_gateway_id {
        Ok(ExitPoint::Gateway {
            identity: NodeIdentity::from_base58_string(exit_router_id.clone())
                .map_err(|_| Error::NodeIdentityFormattingError)?,
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
    return nym_vpn_lib::util::unix_has_root("nym-vpn-cli");

    #[cfg(windows)]
    return nym_vpn_lib::util::win_has_admin("nym-vpn-cli");

    // Assume we're all good on unknown platforms
    debug!("Platform not supported for root privilege check");
    Ok(())
}

async fn run() -> Result<()> {
    setup_logging();
    let args = commands::CliArgs::parse();
    debug!("{:?}", nym_vpn_lib::nym_bin_common::bin_info!());
    setup_env(args.config_env_file.as_ref());

    check_root_privileges(&args)?;

    let data_path = args.data_path.or(mixnet_data_path());

    match args.command {
        Commands::Run(args) => run_vpn(args, data_path).await,
        Commands::ImportCredential(args) => {
            let data_path = data_path.ok_or(Error::ConfigPathNotSet)?;
            import_credential(args, data_path).await
        }
    }
}

async fn run_vpn(args: commands::RunArgs, data_path: Option<PathBuf>) -> Result<()> {
    // Setup gateway configuration
    let gateway_config = override_from_env(&args, GatewayConfig::default());
    info!("nym-api: {}", gateway_config.api_url());
    info!(
        "explorer-api: {}",
        gateway_config
            .explorer_url()
            .map(|url| url.to_string())
            .unwrap_or("unavailable".to_string())
    );

    // Setup wireguard gateway configuration
    let wg_gateway_config = wg_override_from_env(&args, WgGatewayConfig::default());

    let entry_point = parse_entry_point(&args)?;
    let exit_point = parse_exit_point(&args)?;
    let nym_ips = if let (Some(ipv4), Some(ipv6)) = (args.nym_ipv4, args.nym_ipv6) {
        Some(IpPair::new(ipv4, ipv6))
    } else {
        None
    };

    let mut nym_vpn = NymVpn::new(entry_point, exit_point);
    nym_vpn.gateway_config = gateway_config;
    nym_vpn.wg_gateway_config = wg_gateway_config;
    nym_vpn.mixnet_data_path = data_path;
    nym_vpn.enable_wireguard = args.enable_wireguard;
    nym_vpn.private_key = args.private_key;
    nym_vpn.wg_ip = args.wg_ip;
    nym_vpn.nym_ips = nym_ips;
    nym_vpn.nym_mtu = args.nym_mtu;
    nym_vpn.disable_routing = args.disable_routing;
    nym_vpn.enable_two_hop = args.enable_two_hop;
    nym_vpn.enable_poisson_rate = args.enable_poisson_rate;
    nym_vpn.disable_background_cover_traffic = args.disable_background_cover_traffic;
    nym_vpn.enable_credentials_mode = args.enable_credentials_mode;

    nym_vpn.run().await?;

    Ok(())
}

async fn import_credential(args: commands::ImportCredentialArgs, data_path: PathBuf) -> Result<()> {
    info!("Importing credential data into: {}", data_path.display());
    let data: ImportCredentialTypeEnum = args.credential_type.into();
    let raw_credential = match data {
        ImportCredentialTypeEnum::Path(path) => fs::read(path)?,
        ImportCredentialTypeEnum::Data(data) => data,
    };
    fs::create_dir_all(&data_path)?;
    Ok(nym_vpn_lib::credentials::import_credential(raw_credential, data_path).await?)
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
