use once_cell::sync::Lazy;

pub static NETWORK_ENV_SELECT: Lazy<bool> = Lazy::new(|| {
    option_env!("NETWORK_ENV_SELECT")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false)
});
