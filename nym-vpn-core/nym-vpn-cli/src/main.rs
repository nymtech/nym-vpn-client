// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod commands;

use std::fs;
use std::path::PathBuf;

use commands::{CliArgs, ImportCredentialTypeEnum};
use nym_vpn_lib::gateway_directory::{Config as GatewayConfig, EntryPoint, ExitPoint};
use nym_vpn_lib::nym_bin_common::bin_info;
use nym_vpn_lib::{
    error::*, GenericNymVpnConfig, IpPair, MixnetClientConfig, NodeIdentity, SpecificVpn,
};
use nym_vpn_lib::{NymVpn, Recipient};
use time::OffsetDateTime;

use crate::commands::Commands;
use clap::Parser;
use log::*;
use nym_vpn_lib::nym_config::defaults::{setup_env, var_names};

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
    let args = commands::CliArgs::parse();
    setup_logging(&args);
    debug!("{:?}", nym_vpn_lib::nym_bin_common::bin_info!());
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
    let gateway_config = GatewayConfig::new_from_env();
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
            min_gateway_performance: None,
        },
        data_path,
        gateway_config,
        entry_point: entry_point.clone(),
        exit_point: exit_point.clone(),
        enable_two_hop: args.enable_two_hop,
        nym_ips,
        nym_mtu: args.nym_mtu,
        dns: args.dns,
        disable_routing: args.disable_routing,
        user_agent: Some(bin_info!().into()),
    };

    let mut nym_vpn: SpecificVpn = if args.wireguard_mode {
        let mut nym_vpn = NymVpn::new_wireguard_vpn(entry_point, exit_point);
        nym_vpn.generic_config = generic_config;
        nym_vpn.into()
    } else {
        let mut nym_vpn = NymVpn::new_mixnet_vpn(entry_point, exit_point);
        nym_vpn.generic_config = generic_config;
        nym_vpn.into()
    };

    nym_vpn.run().await?;

    Ok(())
}

async fn import_credential(
    args: commands::ImportCredentialArgs,
    data_path: PathBuf,
) -> Result<Option<OffsetDateTime>> {
    info!("Importing credential data into: {}", data_path.display());
    let data: ImportCredentialTypeEnum = args.credential_type.into();
    let raw_credential = match data {
        ImportCredentialTypeEnum::Path(path) => fs::read(path)?,
        ImportCredentialTypeEnum::Data(data) => parse_encoded_credential_data(&data)?,
    };
    fs::create_dir_all(&data_path)?;
    Ok(nym_vpn_lib::credentials::import_credential(raw_credential, data_path).await?)
}

fn parse_encoded_credential_data(raw: &str) -> bs58::decode::Result<Vec<u8>> {
    bs58::decode(raw).into_vec()
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
