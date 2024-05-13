use std::{path::PathBuf, sync::Arc};

use clap::Parser;
use serde::{Deserialize, Serialize};
use tauri::PackageInfo;

pub type ManagedCli = Arc<Cli>;

// generate `crate::build_info` function that returns the data
// collected during build time
// see https://github.com/danielschemmel/build-info
// build_info::build_info!(fn build_info);

#[derive(Parser, Serialize, Deserialize, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Disable the splash-screen
    #[arg(short, long)]
    pub nosplash: bool,

    /// Print build information
    #[arg(short, long)]
    pub build_info: bool,

    /// Sandbox network
    #[arg(short, long)]
    pub sandbox: bool,

    /// Unix socket path of gRPC endpoint in IPC mode
    #[arg(short, long)]
    pub grpc_socket_endpoint: Option<PathBuf>,

    /// Enable HTTP transport for gRPC connection
    #[arg(short = 'H', long)]
    pub grpc_http_mode: bool,

    /// Address of gRPC endpoint in HTTP mode
    #[arg(short = 'e', long)]
    pub grpc_http_endpoint: Option<String>,

    /// IP address of the DNS server to use when connected to the VPN
    #[arg(short = 'D', long)]
    pub dns: Option<String>,
}

// TODO restore this
pub fn print_build_info(_package_info: &PackageInfo) {
    // let info = build_info();

    //     print!(
    //         r"crate name:      {}
    // version:         {}
    // tauri version:   {}
    // package name:    {}
    // package version: {}
    // target:          {}
    // profile:         {}
    // build date:      {}
    // rustc version:   {}
    // rustc channel:   {}
    // ",
    //         info.crate_info.name,
    //         info.crate_info.version,
    //         tauri::VERSION,
    //         package_info.name,
    //         package_info.version,
    //         info.target.triple,
    //         info.profile,
    //         info.timestamp,
    //         info.compiler.version,
    //         info.compiler.channel,
    //     );
    //     if let Some(git) = info.version_control.as_ref().and_then(|vc| vc.git()) {
    //         println!(
    //             r"commit sha:      {}
    // commit date:     {}
    // git branch:      {}
    // ",
    //             git.commit_id,
    //             git.commit_timestamp,
    //             git.branch.as_ref().unwrap_or(&"".to_string())
    //         );
    //     }
}
