use crate::db::{Db, Key};
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use strum::IntoEnumIterator;
use tauri::PackageInfo;
use tracing::{error, info};

#[cfg(all(not(debug_assertions), windows))]
const CONSOLE_FLAGS: [&str; 8] = [
    "-h",
    "--help",
    "-V",
    "--version",
    "-b",
    "--build-info",
    "help",
    "db",
];

/// In release mode on Windows the app is configured as a GUI app so
/// Windows won't attach a console window to it. In order to see
/// output of CLI arguments like `help` or `version` this function
/// attaches a console to the parent process when needed.
// see https://github.com/tauri-apps/tauri/issues/8305#issuecomment-1826871949
#[cfg(all(not(debug_assertions), windows))]
pub fn attach_console() {
    if std::env::args().any(|arg| CONSOLE_FLAGS.contains(&arg.as_str())) {
        {
            use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
            let _ = unsafe { AttachConsole(ATTACH_PARENT_PROCESS) };
            println!();
        }
    }
}

#[derive(
    Parser, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, ValueEnum, strum::Display,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Parser, Serialize, Deserialize, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Print build information
    #[arg(short, long)]
    pub build_info: bool,

    /// Unix socket path of gRPC endpoint
    #[arg(short, long)]
    pub grpc_socket_endpoint: Option<PathBuf>,

    /// IP address of the DNS server to use when connected to the VPN
    #[arg(short = 'D', long)]
    pub dns: Option<String>,

    /// Enable writing app logs to a file
    #[arg(short, long)]
    pub log_file: bool,

    /// Set the log level
    #[arg(short = 'L', long)]
    pub log_level: Option<LogLevel>,

    /// Open a console to see the logs
    #[arg(short, long)]
    #[cfg(windows)]
    pub console: bool,

    /// Disable the splash-screen
    #[arg(short = 's', long)]
    pub nosplash: bool,

    // Run in 'dev' mode
    #[arg(long, hide = true)]
    pub dev_mode: bool,

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
    /// List all keys
    Keys,
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

pub fn db_command(command: &DbCommands) -> Result<()> {
    let db = Db::new().inspect_err(|e| {
        error!("failed to get db: {e}");
    })?;

    match command {
        DbCommands::Keys => {
            info!("cli db keys");
            for key in Key::iter() {
                println!("{key}");
            }
            Ok(())
        }
        DbCommands::Get { key: k } => {
            info!("cli db get {k}");
            if let Some(value) = db.get(k)? {
                println!("{value}");
            } else {
                println!("key is not set");
            }
            Ok(())
        }
        DbCommands::Set { key: k, value: v } => {
            info!("cli db set {k} {v}");
            let value: Value = serde_json::from_str(v).map_err(|e| {
                error!("failed to deserialize json value: {e}");
                anyhow!("invalid value")
            })?;
            db.insert(k, value)?;
            println!("key set to {v}");
            Ok(())
        }
        DbCommands::Del { key: k } => {
            info!("cli db del {k}");
            if let Some(value) = db.remove(k)? {
                println!("key removed, previous value {value}");
            } else {
                println!("key is not set");
            }
            Ok(())
        }
    }
}

pub fn print_build_info(package_info: &PackageInfo) {
    let info = crate::build_info();

    print!(
        r"name:          {}
version:       {}
tauri version: {}
target:        {}
profile:       {}
build date:    {}
rustc version: {}
rustc channel: {}
",
        package_info.name,
        package_info.version,
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
    }
    println!();
}
