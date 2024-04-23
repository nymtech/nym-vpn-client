// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use nym_vpn_proto::{
    nym_vpnd_client::NymVpndClient, ConnectRequest, DisconnectRequest, ImportUserCredentialRequest,
    StatusRequest,
};
use parity_tokio_ipc::Endpoint as IpcEndpoint;
use tonic::transport::{Channel as TonicChannel, Endpoint as TonicEndpoint};

#[derive(Parser)]
#[clap(author = "Nymtech", version, about)]
struct CliArgs {
    /// Use HTTP instead of socket file for IPC with the daemon.
    #[arg(long)]
    http: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Connect,
    Disconnect,
    Status,
    ImportCredential(ImportCredentialArgs),
}

#[derive(Args)]
pub(crate) struct ImportCredentialArgs {
    #[command(flatten)]
    pub(crate) credential_type: ImportCredentialType,

    // currently hidden as there exists only a single serialization standard
    #[arg(long, hide = true)]
    pub(crate) version: Option<u8>,
}

#[derive(Args, Clone)]
#[group(required = true, multiple = false)]
pub(crate) struct ImportCredentialType {
    /// Credential encoded using base58.
    #[arg(long)]
    pub(crate) credential_data: Option<String>,

    /// Path to the credential file.
    #[arg(long)]
    pub(crate) credential_path: Option<PathBuf>,
}

fn parse_encoded_credential_data(raw: &str) -> bs58::decode::Result<Vec<u8>> {
    bs58::decode(raw).into_vec()
}

// Workaround until clap supports enums for ArgGroups
pub(crate) enum ImportCredentialTypeEnum {
    Path(PathBuf),
    Data(String),
}

impl From<ImportCredentialType> for ImportCredentialTypeEnum {
    fn from(ict: ImportCredentialType) -> Self {
        match (ict.credential_data, ict.credential_path) {
            (Some(data), None) => ImportCredentialTypeEnum::Data(data),
            (None, Some(path)) => ImportCredentialTypeEnum::Path(path),
            _ => unreachable!(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    match args.command {
        Command::Connect => connect(&args).await?,
        Command::Disconnect => disconnect(&args).await?,
        Command::Status => status(&args).await?,
        Command::ImportCredential(ref import_args) => import_credential(&args, import_args).await?,
    }
    Ok(())
}

fn get_socket_path() -> PathBuf {
    Path::new("/var/run/nym-vpn.sock").to_path_buf()
}

async fn get_channel(socket_path: PathBuf) -> anyhow::Result<TonicChannel> {
    // NOTE: the uri here is ignored
    Ok(TonicEndpoint::from_static("http://[::1]:53181")
        .connect_with_connector(tower::service_fn(move |_| {
            IpcEndpoint::connect(socket_path.clone())
        }))
        .await?)
}

fn default_endpoint() -> String {
    "http://[::1]:53181".to_string()
}

async fn get_client(args: &CliArgs) -> anyhow::Result<NymVpndClient<TonicChannel>> {
    if args.http {
        let endpoint = default_endpoint();
        let client = NymVpndClient::connect(endpoint.clone())
            .await
            .with_context(|| format!("Failed to connect to: {}", endpoint))?;
        Ok(client)
    } else {
        let socket_path = get_socket_path();
        let channel = get_channel(socket_path.clone())
            .await
            .with_context(|| format!("Failed to connect to: {:?}", socket_path))?;
        let client = NymVpndClient::new(channel);
        Ok(client)
    }
}

fn new_entry_node_location(country_code: &str) -> nym_vpn_proto::Node {
    nym_vpn_proto::Node {
        node_enum: Some(nym_vpn_proto::node::NodeEnum::Location(
            nym_vpn_proto::Location {
                two_letter_iso_country_code: country_code.to_string(),
            },
        )),
    }
}

async fn connect(args: &CliArgs) -> anyhow::Result<()> {
    let mut client = get_client(args).await?;
    let country_code = "DE";
    let request = tonic::Request::new(ConnectRequest {
        entry: Some(new_entry_node_location(country_code)),
    });
    let response = client.vpn_connect(request).await?.into_inner();
    println!("{:?}", response);
    Ok(())
}

async fn disconnect(args: &CliArgs) -> anyhow::Result<()> {
    let mut client = get_client(args).await?;
    let request = tonic::Request::new(DisconnectRequest {});
    let response = client.vpn_disconnect(request).await?.into_inner();
    println!("{:?}", response);
    Ok(())
}

async fn status(args: &CliArgs) -> anyhow::Result<()> {
    let mut client = get_client(args).await?;
    let request = tonic::Request::new(StatusRequest {});
    let response = client.vpn_status(request).await?.into_inner();
    println!("{:?}", response);
    Ok(())
}

async fn import_credential(
    args: &CliArgs,
    import_args: &ImportCredentialArgs,
) -> anyhow::Result<()> {
    let import_type: ImportCredentialTypeEnum = import_args.credential_type.clone().into();
    let raw_credential = match import_type {
        ImportCredentialTypeEnum::Path(path) => std::fs::read(path)?,
        ImportCredentialTypeEnum::Data(data) => parse_encoded_credential_data(&data)?,
    };
    let request = tonic::Request::new(ImportUserCredentialRequest {
        credential: raw_credential,
    });
    let mut client = get_client(args).await?;
    let response = client.import_user_credential(request).await?.into_inner();
    println!("{:?}", response);
    Ok(())
}
