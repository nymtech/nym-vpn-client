// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;
use nym_gateway_directory::GatewayType;
use nym_vpn_proto::{
    ConnectRequest, DisconnectRequest, Empty, ImportUserCredentialRequest, InfoRequest,
    ListCountriesRequest, ListGatewaysRequest, StatusRequest, StoreAccountRequest,
};
use protobuf_conversion::{into_gateway_type, into_threshold};
use vpnd_client::ClientType;

use crate::{
    cli::{Command, ImportCredentialTypeEnum},
    protobuf_conversion::{
        into_entry_point, into_exit_point, ipaddr_into_string, parse_offset_datetime,
    },
};

mod cli;
mod config;
mod protobuf_conversion;
mod vpnd_client;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::CliArgs::parse();
    let client_type = if args.http {
        vpnd_client::ClientType::Http
    } else {
        vpnd_client::ClientType::Ipc
    };
    match args.command {
        Command::Connect(ref connect_args) => connect(client_type, connect_args).await?,
        Command::Disconnect => disconnect(client_type).await?,
        Command::Status => status(client_type).await?,
        Command::Info => info(client_type).await?,
        Command::ImportCredential(ref import_args) => {
            import_credential(client_type, import_args).await?
        }
        Command::StoreAccount(ref store_args) => store_account(client_type, store_args).await?,
        Command::ListenToStatus => listen_to_status(client_type).await?,
        Command::ListenToStateChanges => listen_to_state_changes(client_type).await?,
        Command::ListEntryGateways(ref list_args) => {
            list_gateways(client_type, list_args, GatewayType::Entry).await?
        }
        Command::ListExitGateways(ref list_args) => {
            list_gateways(client_type, list_args, GatewayType::Exit).await?
        }
        Command::ListVpnGateways(ref list_args) => {
            list_gateways(client_type, list_args, GatewayType::Vpn).await?
        }
        Command::ListEntryCountries(ref list_args) => {
            list_countries(client_type, list_args, GatewayType::Entry).await?
        }
        Command::ListExitCountries(ref list_args) => {
            list_countries(client_type, list_args, GatewayType::Exit).await?
        }
        Command::ListVpnCountries(ref list_args) => {
            list_countries(client_type, list_args, GatewayType::Vpn).await?
        }
    }
    Ok(())
}

async fn connect(client_type: ClientType, connect_args: &cli::ConnectArgs) -> Result<()> {
    let entry = cli::parse_entry_point(connect_args)?;
    let exit = cli::parse_exit_point(connect_args)?;

    let request = tonic::Request::new(ConnectRequest {
        entry: entry.map(into_entry_point),
        exit: exit.map(into_exit_point),
        dns: connect_args.dns.map(ipaddr_into_string),
        disable_routing: connect_args.disable_routing,
        enable_two_hop: connect_args.enable_two_hop,
        enable_poisson_rate: connect_args.enable_poisson_rate,
        disable_background_cover_traffic: connect_args.disable_background_cover_traffic,
        enable_credentials_mode: connect_args.enable_credentials_mode,
        min_mixnode_performance: connect_args.min_mixnode_performance.map(into_threshold),
        min_gateway_mixnet_performance: connect_args
            .min_gateway_mixnet_performance
            .map(into_threshold),
        min_gateway_vpn_performance: connect_args.min_gateway_vpn_performance.map(into_threshold),
    });

    let mut client = vpnd_client::get_client(client_type).await?;
    let response = client.vpn_connect(request).await?.into_inner();
    println!("{:#?}", response);
    Ok(())
}

async fn disconnect(client_type: ClientType) -> Result<()> {
    let mut client = vpnd_client::get_client(client_type).await?;
    let request = tonic::Request::new(DisconnectRequest {});
    let response = client.vpn_disconnect(request).await?.into_inner();
    println!("{:#?}", response);
    Ok(())
}

async fn status(client_type: ClientType) -> Result<()> {
    let mut client = vpnd_client::get_client(client_type).await?;
    let request = tonic::Request::new(StatusRequest {});
    let response = client.vpn_status(request).await?.into_inner();
    println!("{:#?}", response);

    if let Some(Ok(utc_since)) = response
        .details
        .and_then(|details| details.since)
        .map(parse_offset_datetime)
    {
        println!("since (utc): {:?}", utc_since);
        println!("duration: {}", time::OffsetDateTime::now_utc() - utc_since);
    }

    Ok(())
}

async fn info(client_type: ClientType) -> Result<()> {
    let mut client = vpnd_client::get_client(client_type).await?;
    let request = tonic::Request::new(InfoRequest {});
    let response = client.info(request).await?.into_inner();
    println!("{:#?}", response);

    if let Some(Ok(utc_build_timestamp)) = response.build_timestamp.map(parse_offset_datetime) {
        println!("build timestamp (utc): {:?}", utc_build_timestamp);
        println!(
            "build age: {}",
            time::OffsetDateTime::now_utc() - utc_build_timestamp
        );
    }
    Ok(())
}

async fn import_credential(
    client_type: ClientType,
    import_args: &cli::ImportCredentialArgs,
) -> Result<()> {
    let import_type: ImportCredentialTypeEnum = import_args.credential_type.clone().into();
    let raw_credential = match import_type {
        ImportCredentialTypeEnum::Path(path) => std::fs::read(path)?,
        ImportCredentialTypeEnum::Data(data) => parse_encoded_credential_data(&data)?,
    };
    let request = tonic::Request::new(ImportUserCredentialRequest {
        credential: raw_credential,
    });
    let mut client = vpnd_client::get_client(client_type).await?;
    let response = client.import_user_credential(request).await?.into_inner();
    println!("{:#?}", response);
    Ok(())
}

fn parse_encoded_credential_data(raw: &str) -> bs58::decode::Result<Vec<u8>> {
    bs58::decode(raw).into_vec()
}

async fn store_account(client_type: ClientType, store_args: &cli::StoreAccountArgs) -> Result<()> {
    let mut client = vpnd_client::get_client(client_type).await?;
    let request = tonic::Request::new(StoreAccountRequest {
        mnemonic: store_args.mnemonic.clone(),
        nonce: 0,
    });
    let response = client.store_account(request).await?.into_inner();
    println!("{:#?}", response);
    Ok(())
}

async fn listen_to_status(client_type: ClientType) -> Result<()> {
    let mut client = vpnd_client::get_client(client_type).await?;
    let request = tonic::Request::new(Empty {});
    let mut stream = client
        .listen_to_connection_status(request)
        .await?
        .into_inner();
    while let Some(response) = stream.message().await? {
        println!("{:#?}", response);
    }
    Ok(())
}

async fn listen_to_state_changes(client_type: ClientType) -> Result<()> {
    let mut client = vpnd_client::get_client(client_type).await?;
    let request = tonic::Request::new(Empty {});
    let mut stream = client
        .listen_to_connection_state_changes(request)
        .await?
        .into_inner();
    while let Some(response) = stream.message().await? {
        println!("{:#?}", response);
    }
    Ok(())
}

async fn list_gateways(
    client_type: ClientType,
    list_args: &cli::ListGatewaysArgs,
    gw_type: GatewayType,
) -> Result<()> {
    let mut client = vpnd_client::get_client(client_type).await?;
    let request = tonic::Request::new(ListGatewaysRequest {
        kind: into_gateway_type(gw_type) as i32,
        min_mixnet_performance: list_args.min_mixnet_performance.map(into_threshold),
        min_vpn_performance: list_args.min_vpn_performance.map(into_threshold),
    });
    let response = client.list_gateways(request).await?.into_inner();
    println!("{:#?}", response);

    if list_args.verbose {
        for gateway in response.gateways {
            let id = gateway.id.unwrap();
            let last_probe = gateway.last_probe.unwrap();
            let last_updated_utc = parse_offset_datetime(last_probe.last_updated_utc.unwrap());
            println!("id: {:?}, last_updated_utc: {:?}", id, last_updated_utc);
        }
    }
    Ok(())
}

async fn list_countries(
    client_type: ClientType,
    list_args: &cli::ListCountriesArgs,
    gw_type: GatewayType,
) -> Result<()> {
    let mut client = vpnd_client::get_client(client_type).await?;
    let request = tonic::Request::new(ListCountriesRequest {
        kind: into_gateway_type(gw_type) as i32,
        min_mixnet_performance: list_args.min_mixnet_performance.map(into_threshold),
        min_vpn_performance: list_args.min_vpn_performance.map(into_threshold),
    });
    let response = client.list_countries(request).await?.into_inner();
    println!("{:#?}", response);
    Ok(())
}
