use once_cell::sync::Lazy;
use std::path::PathBuf;
use tauri::api::path::cache_dir;
use tracing::error;

use crate::fs::util::check_dir;
use crate::APP_DIR;

pub const LOG_DIR: &str = "log";

pub static LOG_DIR_PATH: Lazy<Option<PathBuf>> = Lazy::new(|| {
    let log_dir = cache_dir()
        .map(|mut p| {
            p.push(APP_DIR);
            p.push(LOG_DIR);
            p
        })
        .and_then(|p| check_dir(&p).map(|_| p).ok());

    if log_dir.is_none() {
        error!("Failed to get the log directory");
    }
    log_dir
});
