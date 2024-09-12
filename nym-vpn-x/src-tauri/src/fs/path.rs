use crate::fs::util::check_dir;
use crate::APP_DIR;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use tracing::{debug, error, info};

pub const LOG_DIR: &str = "log";

/// The app config directory
/// - Linux: `$XDG_CONFIG_HOME/nymvpn-x` or `$HOME/.config/nymvpn-x`
/// - Windows: `{FOLDERID_RoamingAppData}\nymvpn-x`, example  `C:\Users\Pierre\AppData\Roaming\nymvpn-x`
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
            .inspect(|p| info!("app config dir: {}", p.display()))
            .ok()
    } else {
        error!("failed to get config dir");
        None
    }
});

/// The app data directory
/// - Linux: `$XDG_DATA_HOME/nymvpn-x` or `$HOME/.local/share/nymvpn-x`
/// - Windows: `{FOLDERID_RoamingAppData}\nymvpn-x`, example `C:\Users\Pierre\AppData\Roaming\nymvpn-x`
pub static APP_DATA_DIR: Lazy<Option<PathBuf>> = Lazy::new(|| {
    let path = dirs::data_dir().map(|mut p| {
        p.push(APP_DIR);
        p
    });
    if let Some(p) = path {
        debug!("checking app data dir: {}", p.display());
        check_dir(&p)
            .inspect_err(|e| error!("failed to check data dir {}: {}", p.display(), e))
            .map(|_| p)
            .inspect(|p| info!("app data dir: {}", p.display()))
            .ok()
    } else {
        error!("failed to get config dir");
        None
    }
});

/// The app log directory
/// - Linux: `$XDG_CACHE_HOME/nymvpn-x/log` or `$HOME/.cache/nymvpn-x/log`
/// - Windows: `{FOLDERID_LocalAppData}\nymvpn-x\log`, example `C:\Users\Pierre\AppData\Local\nymvpn-x\log`
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
            .inspect(|p| info!("app log dir: {}", p.display()))
            .ok()
    } else {
        error!("failed to get log dir");
        None
    }
});
