use nym_vpn_lib::{
    gateway_directory::{EntryPoint, ExitPoint},
    nym_config::defaults::setup_env,
};
use std::path::PathBuf;
use tracing::*;

use nym_ip_pinger::PingResult;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(err) = run().await {
        error!("Exit with error: {err}");
        eprintln!("An error occurred: {err}");
        std::process::exit(1)
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

async fn run() -> anyhow::Result<PingResult> {
    setup_logging();
    debug!("{:?}", nym_vpn_lib::nym_bin_common::bin_info!());
    // mainnet by default
    setup_env::<PathBuf>(None);
    let result = nym_ip_pinger::ping(EntryPoint::Random, ExitPoint::Random).await;
    match result {
        Ok(ref result) => {
            println!("{:#?}", result);
        }
        Err(ref err) => {
            println!("Error: {err}");
        }
    };
    result
}
