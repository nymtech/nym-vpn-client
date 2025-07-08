use once_cell::sync::Lazy;
use std::env;

// compile-time environment variables
/// SemVer version requirement for daemon compatibility
pub const VPND_COMPAT_REQ: Option<&str> = option_env!("VPND_COMPAT_REQ");
#[cfg(windows)]
pub const UPDATER_ENDPOINT: Option<&str> = option_env!("UPDATER_ENDPOINT");

pub static DEV_MODE: Lazy<bool> = Lazy::new(|| {
    option_env!("DEV_MODE")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
});
pub static UPDATER_ENABLED: Lazy<bool> = Lazy::new(|| {
    option_env!("UPDATER_ENABLED")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
});
pub static SENTRY_DSN: Lazy<Option<String>> = Lazy::new(|| {
    env::var("SENTRY_DSN")
        .ok()
        .or_else(|| option_env!("SENTRY_DSN").map(|s| s.to_string()))
});

/// Check if an environment variable is truthy, e.g. set to "1" | "true" | "TRUE"
pub fn is_truthy(var: &str) -> bool {
    match env::var(var) {
        Ok(val) => val == "1" || val.to_lowercase() == "true",
        Err(_) => false,
    }
}
