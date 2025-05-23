use once_cell::sync::Lazy;
use std::env;

// compile-time environment variables
/// SemVer version requirement for daemon compatibility
pub const VPND_COMPAT_REQ: Option<&str> = option_env!("VPND_COMPAT_REQ");
pub const UPDATER_ENDPOINT: Option<&str> = option_env!("UPDATER_ENDPOINT");

pub static DEV_MODE: Lazy<bool> = Lazy::new(|| {
    option_env!("DEV_MODE")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
});

/// Check if an environment variable is truthy, e.g. set to "1" | "true" | "TRUE"
pub fn is_truthy(var: &str) -> bool {
    match env::var(var) {
        Ok(val) => val == "1" || val.to_lowercase() == "true",
        Err(_) => false,
    }
}
