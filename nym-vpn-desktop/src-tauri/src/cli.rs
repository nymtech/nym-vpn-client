use std::sync::Arc;

use clap::Parser;
use serde::{Deserialize, Serialize};

pub type ManagedCli = Arc<Cli>;

#[derive(Parser, Serialize, Deserialize, Debug, Clone, Copy)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Disable the splash-screen
    #[arg(short, long)]
    pub nosplash: bool,
}
