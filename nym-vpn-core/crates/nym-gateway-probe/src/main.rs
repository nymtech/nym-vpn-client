use std::path::PathBuf;

use anyhow::anyhow;
use clap::Parser;
use nym_config::defaults::setup_env;
use nym_gateway_directory::EntryPoint;
use nym_gateway_probe::ProbeResult;
use tracing::*;

#[derive(Parser)]
#[clap(author, version, about)]
struct CliArgs {
    /// Path pointing to an env file describing the network.
    #[arg(short, long)]
    config_env_file: Option<PathBuf>,

    /// The specific gateway specified by ID.
    #[arg(long, short)]
    gateway: Option<String>,

    /// Disable logging during probe
    #[arg(long, short)]
    no_log: bool,

    /// Arguments to be appended to the wireguard config enabling amnezia-wg configuration
    #[arg(long, short)]
    amnezia_args: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match run().await {
        Ok(ref result) => {
            let json = serde_json::to_string_pretty(result)?;
            println!("{}", json);
        }
        Err(err) => {
            eprintln!("An error occurred: {err}");
            std::process::exit(1)
        }
    }
    Ok(())
}

fn setup_logging() {
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

async fn run() -> anyhow::Result<ProbeResult> {
    let args = CliArgs::parse();
    if !args.no_log {
        setup_logging();
    }
    debug!("{:?}", nym_bin_common::bin_info_local_vergen!());
    setup_env(args.config_env_file.as_ref());

    let gateway = if let Some(gateway) = args.gateway {
        EntryPoint::from_base58_string(&gateway)?
    } else {
        fetch_random_gateway_with_ipr().await?
    };

    let mut trial = nym_gateway_probe::Probe::new(gateway);
    if let Some(awg_args) = args.amnezia_args {
        trial.with_amnezia(&awg_args);
    }
    trial.probe().await
}

async fn fetch_random_gateway_with_ipr() -> anyhow::Result<EntryPoint> {
    // We're fetching gateways with IPR, since they are more interesting to ping, but we can probe
    // gateways without an IPR as well
    let gateways = nym_gateway_probe::fetch_gateways_with_ipr().await?;
    let gateway = gateways
        .random_gateway()
        .ok_or(anyhow!("No gateways returned by nym-api"))?;
    Ok(EntryPoint::Gateway {
        identity: *gateway.identity(),
    })
}
