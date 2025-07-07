// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only
#![warn(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

mod cli;
mod config;
mod protobuf_conversion;
mod vpnd_client;

use anyhow::{Context, Result, bail};
use clap::Parser;
use cli::Internal;
use itertools::Itertools;
use nym_gateway_directory::GatewayType;
use nym_vpn_lib_types::TunnelState;
use nym_vpn_network_config::ParsedAccountLinks;
use nym_vpn_proto::{
    ConfirmZkNymDownloadedRequest, ConnectRequest, GetAccountLinksRequest, GetZkNymByIdRequest,
    InfoResponse, ListCountriesRequest, ListGatewaysRequest, ResetDeviceIdentityRequest,
    StoreAccountRequest, UserAgent,
};
use protobuf_conversion::into_gateway_type;
use sysinfo::System;

use crate::{
    cli::Command,
    protobuf_conversion::{into_entry_point, into_exit_point},
};

#[derive(Clone, Debug)]
struct CliOptions {
    verbose: bool,
    user_agent: Option<nym_http_api_client::UserAgent>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::CliArgs::parse();
    let opts = CliOptions {
        verbose: args.verbose,
        user_agent: args.user_agent,
    };

    match args.command {
        Command::Connect(ref connect_args) => connect(opts, connect_args).await?,
        Command::Disconnect { wait } => disconnect(opts, wait).await?,
        Command::Status { listen } => status(listen).await?,
        Command::Info => info().await?,
        Command::SetNetwork(ref args) => set_network(args).await?,
        Command::StoreAccount(ref store_args) => store_account(opts, store_args).await?,
        Command::IsAccountStored => is_account_stored().await?,
        Command::ForgetAccount => forget_account().await?,
        Command::GetAccountId => get_account_id().await?,
        Command::GetAccountLinks(ref args) => get_account_links(opts, args).await?,
        Command::GetAccountState => get_account_state().await?,
        Command::ListEntryGateways(ref list_args) => {
            list_gateways(opts, list_args, GatewayType::MixnetEntry).await?
        }
        Command::ListExitGateways(ref list_args) => {
            list_gateways(opts, list_args, GatewayType::MixnetExit).await?
        }
        Command::ListVpnGateways(ref list_args) => {
            list_gateways(opts, list_args, GatewayType::Wg).await?
        }
        Command::ListEntryCountries(ref list_args) => {
            list_countries(opts, list_args, GatewayType::MixnetEntry).await?
        }
        Command::ListExitCountries(ref list_args) => {
            list_countries(opts, list_args, GatewayType::MixnetExit).await?
        }
        Command::ListVpnCountries(ref list_args) => {
            list_countries(opts, list_args, GatewayType::Wg).await?
        }
        Command::GetDeviceId => get_device_id().await?,
        Command::Internal(internal) => match internal {
            Internal::GetSystemMessages => get_system_messages().await?,
            Internal::GetFeatureFlags => get_feature_flags().await?,
            Internal::SyncAccountState => refresh_account_state().await?,
            Internal::GetAccountUsage => get_account_usage().await?,
            Internal::ResetDeviceIdentity(ref args) => reset_device_identity(args).await?,
            Internal::RegisterDevice => register_device().await?,
            Internal::GetDevices => get_devices().await?,
            Internal::GetActiveDevices => get_active_devices().await?,
            Internal::RequestZkNym => request_zk_nym().await?,
            Internal::GetDeviceZkNym => get_device_zk_nym().await?,
            Internal::GetZkNymsAvailableForDownload => get_zk_nyms_available_for_download().await?,
            Internal::GetZkNymById(args) => get_zk_nym_by_id(args).await?,
            Internal::ConfirmZkNymDownloaded(args) => confirm_zk_nym_downloaded(args).await?,
            Internal::GetAvailableTickets => get_available_tickets().await?,
        },
    }
    Ok(())
}

fn setup_user_agent(opts: &CliOptions, daemon_info: InfoResponse) -> UserAgent {
    opts.user_agent
        .clone()
        .map(nym_vpn_proto::UserAgent::from)
        .unwrap_or_else(|| construct_user_agent(daemon_info))
}

fn construct_user_agent(daemon_info: InfoResponse) -> UserAgent {
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

async fn connect(opts: CliOptions, connect_args: &cli::ConnectArgs) -> Result<()> {
    let entry = cli::parse_entry_point(connect_args)?;
    let exit = cli::parse_exit_point(connect_args)?;

    let mut client = vpnd_client::get_client().await?;
    let info = client.info(()).await?.into_inner();
    let user_agent = setup_user_agent(&opts, info);

    let request = tonic::Request::new(ConnectRequest {
        entry: entry.map(into_entry_point),
        exit: exit.map(into_exit_point),
        dns: connect_args.dns.map(nym_vpn_proto::Dns::from),
        enable_two_hop: connect_args.enable_two_hop,
        netstack: connect_args.netstack,
        disable_poisson_rate: connect_args.disable_poisson_rate,
        disable_background_cover_traffic: connect_args.disable_background_cover_traffic,
        enable_credentials_mode: connect_args.enable_credentials_mode,
        user_agent: Some(user_agent),
    });

    let connect_accepted = client.vpn_connect(request).await?.into_inner();

    if opts.verbose {
        println!("{connect_accepted:#?}");
    }

    if connect_accepted {
        if connect_args.wait {
            println!("Waiting until connected or failed");
            wait_until_connected().await
        } else {
            Ok(())
        }
    } else {
        println!("Connect has not been accepted");
        Ok(())
    }
}

async fn wait_until_connected() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;

    let mut stream = client.listen_to_tunnel_state(()).await?.into_inner();
    while let Some(new_state) = stream.message().await? {
        let new_state = TunnelState::try_from(new_state)?;
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
                bail!("Tunnel entered error state {:?}", reason);
            }
            _ => {}
        }
    }
    Ok(())
}

async fn disconnect(opts: CliOptions, wait: bool) -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.vpn_disconnect(()).await?.into_inner();

    if opts.verbose {
        println!("{response:#?}");
    }

    if response {
        if wait {
            println!("Successfully sent disconnect command. Waiting until disconnected.");
            wait_until_disconnected().await
        } else {
            println!("Successfully sent disconnect command");
            Ok(())
        }
    } else {
        println!("Disconnect command failed");
        Ok(())
    }
}

async fn wait_until_disconnected() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let mut stream = client.listen_to_tunnel_state(()).await?.into_inner();
    while let Some(new_state) = stream.message().await? {
        let new_state = TunnelState::try_from(new_state)?;
        println!("{new_state}");

        match new_state {
            TunnelState::Disconnected | TunnelState::Offline { .. } => {
                break;
            }
            TunnelState::Error(reason) => {
                bail!("Tunnel entered error state: {:?}", reason)
            }
            _ => {}
        }
    }
    Ok(())
}

async fn status(listen: bool) -> Result<()> {
    let mut client = vpnd_client::get_client().await?;

    if listen {
        let mut stream = client.listen_to_tunnel_state(()).await?.into_inner();
        while let Some(new_state) = stream.message().await? {
            let tunnel_state = TunnelState::try_from(new_state)?;
            println!("{tunnel_state}");
        }
    } else {
        let tunnel_state = TunnelState::try_from(client.get_tunnel_state(()).await?.into_inner())?;
        println!("{tunnel_state}");
    }

    Ok(())
}

async fn info() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.info(()).await?.into_inner();
    let info = nym_vpn_proto::conversions::InfoResponse::try_from(response)
        .context("failed to parse info response")?;
    println!("{info}");
    Ok(())
}

async fn set_network(args: &cli::SetNetworkArgs) -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.set_network(args.network.clone()).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn get_system_messages() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.get_system_messages(()).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn get_feature_flags() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.get_feature_flags(()).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn store_account(opts: CliOptions, store_args: &cli::StoreAccountArgs) -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let request = tonic::Request::new(StoreAccountRequest {
        mnemonic: store_args.mnemonic.clone(),
    });
    let response = client.store_account(request).await?.into_inner();
    if opts.verbose {
        println!("{response:#?}");
    }

    if let Some(err) = response
        .error
        .map(nym_vpn_lib_types::StoreAccountError::try_from)
        .map(|err| format!("{err:?}"))
    {
        println!("Error: {err}");
    } else {
        println!("Account recovery phrase stored");
    }

    Ok(())
}

async fn refresh_account_state() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.refresh_account_state(()).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn is_account_stored() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.is_account_stored(()).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn get_account_usage() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.get_account_usage(()).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn forget_account() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.forget_account(()).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn get_account_id() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.get_account_identity(()).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn get_account_links(opts: CliOptions, args: &cli::GetAccountLinksArgs) -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let request = tonic::Request::new(GetAccountLinksRequest {
        locale: args.locale.clone(),
    });
    let response = client.get_account_links(request).await?.into_inner();
    if opts.verbose {
        println!("{response:#?}");
    }

    let links = ParsedAccountLinks::try_from(response)
        .context("failed to parse account management response")?;
    println!("{links}");

    Ok(())
}

async fn get_account_state() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.get_account_state(()).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn reset_device_identity(args: &cli::ResetDeviceIdentityArgs) -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let request = tonic::Request::new(ResetDeviceIdentityRequest {
        seed: args.seed.as_ref().map(|seed| seed.clone().into_bytes()),
    });
    let response = client.reset_device_identity(request).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn get_device_id() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.get_device_identity(()).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn register_device() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.register_device(()).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn get_devices() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.get_devices(()).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn get_active_devices() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.get_active_devices(()).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn request_zk_nym() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.request_zk_nym(()).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn get_device_zk_nym() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.get_device_zk_nyms(()).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn get_zk_nyms_available_for_download() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client
        .get_zk_nyms_available_for_download(())
        .await?
        .into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn get_zk_nym_by_id(args: cli::GetZkNymByIdArgs) -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let request = tonic::Request::new(GetZkNymByIdRequest {
        id: args.id.clone(),
    });
    let response = client.get_zk_nym_by_id(request).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn confirm_zk_nym_downloaded(args: cli::ConfirmZkNymDownloadedArgs) -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let request = tonic::Request::new(ConfirmZkNymDownloadedRequest {
        id: args.id.clone(),
    });
    let response = client
        .confirm_zk_nym_downloaded(request)
        .await?
        .into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn get_available_tickets() -> Result<()> {
    let mut client = vpnd_client::get_client().await?;
    let response = client.get_available_tickets(()).await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn list_gateways(
    opts: CliOptions,
    _list_args: &cli::ListGatewaysArgs,
    gw_type: GatewayType,
) -> Result<()> {
    let mut client = vpnd_client::get_client().await?;

    let info = client.info(()).await?.into_inner();
    let user_agent = setup_user_agent(&opts, info);

    let request = tonic::Request::new(ListGatewaysRequest {
        kind: into_gateway_type(gw_type) as i32,
        user_agent: Some(user_agent),
    });
    let response = client.list_gateways(request).await?.into_inner();
    if opts.verbose {
        println!("{response:#?}");
    }
    println!("Gateways available for: {gw_type}");
    println!("Total gateways: {}", response.gateways.len());
    for gateway in response.gateways.clone() {
        if let Ok(gateway) = nym_vpnd_types::gateway::Gateway::try_from(gateway)
            .inspect_err(|e| println!("Failed to parse gateway: {e}"))
        {
            println!("  {gateway}");
        }
    }
    Ok(())
}

async fn list_countries(
    opts: CliOptions,
    _list_args: &cli::ListCountriesArgs,
    gw_type: GatewayType,
) -> Result<()> {
    let mut client = vpnd_client::get_client().await?;

    let info = client.info(()).await?.into_inner();
    let user_agent = setup_user_agent(&opts, info);

    let request = tonic::Request::new(ListCountriesRequest {
        kind: into_gateway_type(gw_type) as i32,
        user_agent: Some(user_agent),
    });

    let response = client.list_countries(request).await?.into_inner();
    if opts.verbose {
        println!("{response:#?}");
    }

    let countries = response
        .countries
        .into_iter()
        .map(nym_vpnd_types::gateway::Country::from)
        .collect::<Vec<_>>();

    println!(
        "Countries for {} ({}): {}",
        gw_type,
        countries.len(),
        countries.iter().join(", ")
    );

    Ok(())
}
