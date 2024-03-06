use clap::Parser;
use nym_vpn_lib::nym_bin_common::bin_info_local_vergen;
use std::{path::PathBuf, sync::OnceLock};

mod command_interface;
mod service;

// Helper for passing LONG_VERSION to clap
fn pretty_build_info_static() -> &'static str {
    static PRETTY_BUILD_INFORMATION: OnceLock<String> = OnceLock::new();
    PRETTY_BUILD_INFORMATION.get_or_init(|| bin_info_local_vergen!().pretty_print())
}

#[derive(Parser)]
#[clap(author = "Nymtech", version, about, long_version = pretty_build_info_static())]
pub(crate) struct CliArgs {
    /// Path pointing to an env file describing the network.
    #[arg(short, long, value_parser = check_path)]
    pub(crate) config_env_file: Option<PathBuf>,
}

fn check_path(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(format!("Path {:?} does not exist", path));
    }
    if !path.is_file() {
        return Err(format!("Path {:?} is not a file", path));
    }
    Ok(path)
}

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    let args = CliArgs::parse();
    nym_vpn_lib::nym_config::defaults::setup_env(args.config_env_file.as_ref());

    // The idea here for explicly starting two separate runtimes is to make sure they are properly
    // separated. Looking ahead a little ideally it would be nice to be able for the command
    // interface to be able to forcefully terminate the vpn if needed.

    println!("main: starting command handler");
    let (command_handle, vpn_command_rx) = command_interface::start_command_interface();

    println!("main: starting VPN handler");
    let vpn_handle = service::start_vpn_service(vpn_command_rx);

    vpn_handle.join().unwrap();
    command_handle.join().unwrap();

    Ok(())
}
