const DEFAULT_SOCKS5_REQUEST_TIMEOUT_SECS: u64 = 30;
const DEFAULT_SOCKS5_IDLE_TIMEOUT_SECS: u64 = 300;

pub fn socks5_request_timeout() -> std::time::Duration {
    std::env::var("NYM_VPND_SOCKS5_REQUEST_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(std::time::Duration::from_secs)
        .unwrap_or_else(|| std::time::Duration::from_secs(DEFAULT_SOCKS5_REQUEST_TIMEOUT_SECS))
}

pub fn socks5_idle_timeout() -> std::time::Duration {
    std::env::var("NYM_VPND_SOCKS5_IDLE_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(std::time::Duration::from_secs)
        .unwrap_or_else(|| std::time::Duration::from_secs(DEFAULT_SOCKS5_IDLE_TIMEOUT_SECS))
}
