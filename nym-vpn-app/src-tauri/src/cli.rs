use crate::db::{Db, Key};
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::str::FromStr;
use tauri::PackageInfo;
use tracing::{error, info};

#[derive(Parser, Serialize, Deserialize, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Print build information
    #[arg(short, long)]
    pub build_info: bool,

    /// Path to an env file to load a custom network environment
    #[arg(short = 'n', long)]
    pub network_env: Option<PathBuf>,

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

    /// Enable writing app logs to a file
    #[arg(short, long)]
    pub log_file: bool,

    /// Open a console to see the log stream (Windows only)
    #[arg(short, long)]
    pub console: bool,

    /// Disable the splash-screen
    #[arg(short = 's', long)]
    pub nosplash: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Serialize, Deserialize, Debug, Clone)]
pub enum Commands {
    /// Embedded database operations (for debugging purposes only)
    Db {
        #[command(subcommand)]
        command: Option<DbCommands>,
    },
}

#[derive(Subcommand, Serialize, Deserialize, Debug, Clone)]
pub enum DbCommands {
    /// Get a key
    Get {
        #[arg()]
        key: String,
    },
    /// Set a key
    Set {
        #[arg()]
        key: String,
        /// as JSON string
        #[arg()]
        value: String,
    },
    /// Delete a key
    Del {
        #[arg()]
        key: String,
    },
}

pub fn db_command(db: &Db, command: &DbCommands) -> Result<()> {
    match command {
        DbCommands::Get { key: k } => {
            info!("cli db get {k}");
            let key = Key::from_str(k).map_err(|_| anyhow!("invalid key"))?;
            if let Some(value) = db.get(key)? {
                println!("{value}");
            } else {
                println!("key is not set");
            }
            Ok(())
        }
        DbCommands::Set { key: k, value: v } => {
            info!("cli db set {k} {v}");
            let key = Key::from_str(k).map_err(|_| anyhow!("invalid key"))?;
            let value: Value = serde_json::from_str(v).map_err(|e| {
                error!("failed to deserialize json value: {e}");
                anyhow!("invalid value")
            })?;
            db.insert(key, value)?;
            println!("key set to {v}");
            Ok(())
        }
        DbCommands::Del { key: k } => {
            info!("cli db del {k}");
            let key = Key::from_str(k).map_err(|_| anyhow!("invalid key"))?;
            db.remove(key)?;
            println!("key removed");
            Ok(())
        }
    }
}

pub fn print_build_info(package_info: &PackageInfo) {
    let info = crate::build_info();

    print!(
        r"app name:      {}
app version:   {}
crate name:    {}
crate version: {}
tauri version: {}
target:        {}
profile:       {}
build date:    {}
rustc version: {}
rustc channel: {}
",
        package_info.name,
        package_info.version,
        info.crate_info.name,
        info.crate_info.version,
        tauri::VERSION,
        info.target.triple,
        info.profile,
        info.timestamp,
        info.compiler.version,
        info.compiler.channel,
    );
    if let Some(git) = info.version_control.as_ref().and_then(|vc| vc.git()) {
        print!(
            r"commit sha:    {}
commit date:   {}
",
            git.commit_id, git.commit_timestamp,
        );

        if let Some(branch) = git.branch.as_ref() {
            print!("git branch:    {}", branch);
        }
    }
    println!();
}
