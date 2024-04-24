// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use clap::{Args, Parser, Subcommand};
use nym_gateway_directory::{EntryPoint, ExitPoint, NodeIdentity, Recipient};
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
    Connect(ConnectArgs),
    Disconnect,
    Status,
    ImportCredential(ImportCredentialArgs),
}

#[derive(Args)]
pub(crate) struct ConnectArgs {
    #[command(flatten)]
    pub(crate) entry: CliEntry,

    #[command(flatten)]
    pub(crate) exit: CliExit,

    /// Disable routing all traffic through the nym TUN device. When the flag is set, the nym TUN
    /// device will be created, but to route traffic through it you will need to do it manually,
    /// e.g. ping -Itun0.
    #[arg(long)]
    pub(crate) disable_routing: bool,

    /// Enable two-hop mixnet traffic. This means that traffic jumps directly from entry gateway to
    /// exit gateway.
    #[arg(long)]
    pub(crate) enable_two_hop: bool,

    /// Enable Poisson process rate limiting of outbound traffic.
    #[arg(long)]
    pub(crate) enable_poisson_rate: bool,

    /// Disable constant rate background loop cover traffic.
    #[arg(long)]
    pub(crate) disable_background_cover_traffic: bool,

    /// Enable credentials mode.
    #[arg(long)]
    pub(crate) enable_credentials_mode: bool,
}

#[derive(Args)]
#[group(multiple = false)]
pub(crate) struct CliEntry {
    /// Mixnet public ID of the entry gateway.
    #[clap(long, alias = "entry-id")]
    pub(crate) entry_gateway_id: Option<String>,

    /// Auto-select entry gateway by country ISO.
    #[clap(long, alias = "entry-country")]
    pub(crate) entry_gateway_country: Option<String>,

    /// Auto-select entry gateway by latency
    #[clap(long, alias = "entry-fastest")]
    pub(crate) entry_gateway_low_latency: bool,

    /// Auto-select entry gateway randomly.
    #[clap(long, alias = "entry-random")]
    pub(crate) entry_gateway_random: bool,
}

#[derive(Args)]
#[group(multiple = false)]
pub(crate) struct CliExit {
    /// Mixnet recipient address.
    #[clap(long, alias = "exit-address")]
    pub(crate) exit_router_address: Option<String>,

    /// Mixnet public ID of the exit gateway.
    #[clap(long, alias = "exit-id")]
    pub(crate) exit_gateway_id: Option<String>,

    /// Auto-select exit gateway by country ISO.
    #[clap(long, alias = "exit-country")]
    pub(crate) exit_gateway_country: Option<String>,

    /// Auto-select exit gateway randomly.
    #[clap(long, alias = "exit-random")]
    pub(crate) exit_gateway_random: bool,
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

fn parse_entry_point(args: &ConnectArgs) -> anyhow::Result<Option<EntryPoint>> {
    if let Some(ref entry_gateway_id) = args.entry.entry_gateway_id {
        Ok(Some(EntryPoint::Gateway {
            identity: NodeIdentity::from_base58_string(entry_gateway_id.clone())
                .map_err(|_| anyhow!("Failed to parse gateway id"))?,
        }))
    } else if let Some(ref entry_gateway_country) = args.entry.entry_gateway_country {
        Ok(Some(EntryPoint::Location {
            location: entry_gateway_country.clone(),
        }))
    } else if args.entry.entry_gateway_low_latency {
        Ok(Some(EntryPoint::RandomLowLatency))
    } else if args.entry.entry_gateway_random {
        Ok(Some(EntryPoint::Random))
    } else {
        Ok(None)
    }
}

fn parse_exit_point(args: &ConnectArgs) -> anyhow::Result<Option<ExitPoint>> {
    if let Some(ref exit_router_address) = args.exit.exit_router_address {
        Ok(Some(ExitPoint::Address {
            address: Recipient::try_from_base58_string(exit_router_address.clone())
                .map_err(|_| anyhow!("Failed to parse exit node address"))?,
        }))
    } else if let Some(ref exit_router_id) = args.exit.exit_gateway_id {
        Ok(Some(ExitPoint::Gateway {
            identity: NodeIdentity::from_base58_string(exit_router_id.clone())
                .map_err(|_| anyhow!("Failed to parse gateway id"))?,
        }))
    } else if let Some(ref exit_gateway_country) = args.exit.exit_gateway_country {
        Ok(Some(ExitPoint::Location {
            location: exit_gateway_country.clone(),
        }))
    } else if args.exit.exit_gateway_random {
        Ok(Some(ExitPoint::Random))
    } else {
        Ok(None)
    }
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
        Command::Connect(ref connect_args) => connect(&args, connect_args).await?,
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

#[allow(unused)]
fn new_entry_node_location(country_code: &str) -> nym_vpn_proto::EntryNode {
    nym_vpn_proto::EntryNode {
        entry_node_enum: Some(nym_vpn_proto::entry_node::EntryNodeEnum::Location(
            nym_vpn_proto::Location {
                two_letter_iso_country_code: country_code.to_string(),
            },
        )),
    }
}

fn into_entry_point(entry: EntryPoint) -> nym_vpn_proto::EntryNode {
    match entry {
        EntryPoint::Gateway { identity } => nym_vpn_proto::EntryNode {
            entry_node_enum: Some(nym_vpn_proto::entry_node::EntryNodeEnum::Gateway(
                nym_vpn_proto::Gateway {
                    id: identity.to_base58_string(),
                },
            )),
        },
        EntryPoint::Location { location } => nym_vpn_proto::EntryNode {
            entry_node_enum: Some(nym_vpn_proto::entry_node::EntryNodeEnum::Location(
                nym_vpn_proto::Location {
                    two_letter_iso_country_code: location,
                },
            )),
        },
        EntryPoint::RandomLowLatency => nym_vpn_proto::EntryNode {
            entry_node_enum: Some(nym_vpn_proto::entry_node::EntryNodeEnum::RandomLowLatency(
                nym_vpn_proto::Empty {},
            )),
        },
        EntryPoint::Random => nym_vpn_proto::EntryNode {
            entry_node_enum: Some(nym_vpn_proto::entry_node::EntryNodeEnum::Random(
                nym_vpn_proto::Empty {},
            )),
        },
    }
}

fn into_exit_point(exit: ExitPoint) -> nym_vpn_proto::ExitNode {
    match exit {
        ExitPoint::Address { address } => nym_vpn_proto::ExitNode {
            exit_node_enum: Some(nym_vpn_proto::exit_node::ExitNodeEnum::Address(
                nym_vpn_proto::Address {
                    nym_address: address.to_string(),
                },
            )),
        },
        ExitPoint::Gateway { identity } => nym_vpn_proto::ExitNode {
            exit_node_enum: Some(nym_vpn_proto::exit_node::ExitNodeEnum::Gateway(
                nym_vpn_proto::Gateway {
                    id: identity.to_base58_string(),
                },
            )),
        },
        ExitPoint::Location { location } => nym_vpn_proto::ExitNode {
            exit_node_enum: Some(nym_vpn_proto::exit_node::ExitNodeEnum::Location(
                nym_vpn_proto::Location {
                    two_letter_iso_country_code: location,
                },
            )),
        },
        ExitPoint::Random => nym_vpn_proto::ExitNode {
            exit_node_enum: Some(nym_vpn_proto::exit_node::ExitNodeEnum::Random(
                nym_vpn_proto::Empty {},
            )),
        },
    }
}

async fn connect(args: &CliArgs, connect_args: &ConnectArgs) -> anyhow::Result<()> {
    // Setup connect arguments
    let entry = parse_entry_point(connect_args)?;
    let exit = parse_exit_point(connect_args)?;

    let request = tonic::Request::new(ConnectRequest {
        entry: entry.map(into_entry_point),
        exit: exit.map(into_exit_point),
    });

    let mut client = get_client(args).await?;
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
