use std::env;

/// Check if an environment variable is truthy, i.e. set to "1" or "true"
pub fn is_truthy(var: &str) -> bool {
    match env::var(var) {
        Ok(val) => val == "1" || val == "true",
        Err(_) => false,
    }
}
