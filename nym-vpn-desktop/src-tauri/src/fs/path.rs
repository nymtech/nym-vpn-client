use std::path::PathBuf;

use nym_vpn_lib::nym_config::defaults::var_names;
use once_cell::sync::Lazy;
use tauri::api::path::data_dir;
use tracing::{error, trace};

use crate::APP_DIR;

const BACKEND_DIR: &str = "backend";

pub static BACKEND_DATA_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let network_name = std::env::var(var_names::NETWORK_NAME)
        .inspect_err(|e| {
            error!(
                "failed to get current network name: {}, fallback to 'unknown'",
                e
            )
        })
        .unwrap_or("unknown".into());
    let mut path = data_dir().expect("failed to retrieve data dir");
    path.push(APP_DIR);
    path.push(BACKEND_DIR);
    path.push(network_name);
    trace!("using path for backend data: {}", path.to_string_lossy());
    path
});
