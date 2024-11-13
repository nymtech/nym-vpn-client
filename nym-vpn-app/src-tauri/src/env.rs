use once_cell::sync::Lazy;
use std::env;

pub static NETWORK_ENV_SELECT: Lazy<bool> = Lazy::new(|| {
    option_env!("NETWORK_ENV_SELECT")
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
