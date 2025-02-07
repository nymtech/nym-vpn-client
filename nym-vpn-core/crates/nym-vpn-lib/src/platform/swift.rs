use std::{fs::OpenOptions, io::Write, path::PathBuf, str::FromStr};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::filter::LevelFilter;
use tracing_oslog::OsLogger;

pub(crate) const DEFAULT_LOG_FILE: &str = "nym-vpn-lib.log";

pub fn init_logs(level: String, path: Option<PathBuf>) {
    // Set log level
    let log_level = LevelFilter::from_str(&level).unwrap_or(LevelFilter::INFO);

    // Determine log file path
    let log_path = path.unwrap_or_else(|| PathBuf::from(DEFAULT_LOG_FILE));

    // Ensure log directory exists
    if let Some(parent) = log_path.parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("Failed to create log directory {:?}: {}", parent, e);
            }
        }
    }

    // Attempt to open the log file for writing
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&log_path);

    match file {
        Ok(f) => {
            // Initialize the logger with file output
            fmt()
                .with_writer(f)
                .with_max_level(log_level)
                .init();

            tracing::info!("Logger initialized: level = {}, path = {:?}", level, log_path);
        }
        Err(e) => {
            eprintln!("Failed to open log file {:?}: {}. Falling back to os_log.", log_path, e);

            // Initialize fallback logging with `os_log` for macOS/iOS
            let oslogger_layer = OsLogger::new("net.nymtech.vpn.agent", "default");

            tracing_subscriber::registry()
                .with(oslogger_layer)
                .with(log_level)
                .init();

            tracing::info!("Logger initialized with os_log due to file creation failure.");
        }
    }

    // Ensure logs are flushed immediately
    std::io::stdout().flush().unwrap();
    std::io::stderr().flush().unwrap();
}
