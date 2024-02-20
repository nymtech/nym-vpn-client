use std::sync::Arc;

use clap::Parser;
use serde::{Deserialize, Serialize};

pub type ManagedCli = Arc<Cli>;

// generate `crate::build_info` function that returns the data
// collected during build time
// see https://github.com/danielschemmel/build-info
build_info::build_info!(fn build_info);

#[derive(Parser, Serialize, Deserialize, Debug, Clone, Copy)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Disable the splash-screen
    #[arg(short, long)]
    pub nosplash: bool,

    /// Print build information
    #[arg(short, long)]
    pub build_info: bool,
}

pub fn print_build_info() {
    let info = build_info();

    print!(
        r"binary:        {}
version:       {}
target:        {}
profile:       {}
build date:    {}
tauri version: {}
rustc version: {}
rustc channel: {}
",
        info.crate_info.name,
        info.crate_info.version,
        info.target.triple,
        info.profile,
        info.timestamp,
        tauri::VERSION,
        info.compiler.version,
        info.compiler.channel,
    );
    if let Some(git) = info.version_control.as_ref().unwrap().git() {
        println!(
            r"commit sha:    {}
commit date:   {}
git branch:    {}
",
            git.commit_id,
            git.commit_timestamp,
            git.branch.as_ref().unwrap_or(&"".to_string())
        );
    } else {
        println!("git:           not available");
    }
}
