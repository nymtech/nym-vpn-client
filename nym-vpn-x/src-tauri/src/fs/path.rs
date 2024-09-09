use once_cell::sync::Lazy;
use std::path::PathBuf;
use tauri::path::BaseDirectory;

use crate::fs::util::check_dir;
use crate::APP_DIR;

pub const LOG_DIR: &str = "log";

pub static LOG_DIR_PATH: Lazy<Option<PathBuf>> = Lazy::new(|| {
    let path: PathBuf = [BaseDirectory::Cache.variable(), APP_DIR, LOG_DIR]
        .iter()
        .collect();
    check_dir(&path).map(|_| path).ok()
});
