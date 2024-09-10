use crate::fs::util::check_dir;
use crate::APP_DIR;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use tracing::{debug, error};

pub const LOG_DIR: &str = "log";

pub static APP_CONFIG_DIR: Lazy<Option<PathBuf>> = Lazy::new(|| {
    let path = dirs::config_dir().map(|mut p| {
        p.push(APP_DIR);
        p
    });
    if let Some(p) = path {
        debug!("checking app config dir: {}", p.display());
        check_dir(&p)
            .inspect_err(|e| error!("failed to check config dir {}: {}", p.display(), e))
            .map(|_| p)
            .ok()
    } else {
        error!("failed to get config dir");
        None
    }
});

pub static APP_LOG_DIR: Lazy<Option<PathBuf>> = Lazy::new(|| {
    let path = dirs::cache_dir().map(|mut p| {
        p.push(APP_DIR);
        p.push(LOG_DIR);
        p
    });
    if let Some(p) = path {
        debug!("checking app log dir: {}", p.display());
        check_dir(&p)
            .inspect_err(|e| error!("failed to check log dir {}: {}", p.display(), e))
            .map(|_| p)
            .ok()
    } else {
        error!("failed to get log dir");
        None
    }
});
