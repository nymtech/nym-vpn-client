// Copyright 2023-2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fs::OpenOptions, io::Write, path::PathBuf, str::FromStr};

use tracing_oslog::OsLogger;
use tracing_subscriber::{filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub(crate) const DEFAULT_LOG_FILE: &str = "nym-vpn-lib.log";

pub fn init_logs(level: String, path: Option<PathBuf>) {
    // Set log level
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(
            LevelFilter::from_str(&level)
                .unwrap_or(LevelFilter::INFO)
                .into(),
        )
        .from_env()
        .unwrap()
        .add_directive("hyper::proto=warn".parse().unwrap())
        .add_directive("tokio_reactor::proto=warn".parse().unwrap())
        .add_directive("reqwest::proto=warn".parse().unwrap())
        .add_directive("mio::proto=warn".parse().unwrap())
        .add_directive("want::proto=warn".parse().unwrap())
        .add_directive("tungstenite::proto=warn".parse().unwrap())
        .add_directive("tokio_tungstenite::proto=warn".parse().unwrap())
        .add_directive("handlebars::proto=warn".parse().unwrap())
        .add_directive("sled::proto=warn".parse().unwrap());

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
    let file = OpenOptions::new().append(true).create(true).open(&log_path);

    match file {
        Ok(f) => {
            // Initialize the logger with file output
            fmt()
                .compact()
                .with_writer(f)
                .with_env_filter(filter)
                .with_ansi(false)
                .init();

            tracing::info!(
                "Logger initialized: level = {}, path = {:?}",
                level,
                log_path
            );
        }
        Err(e) => {
            eprintln!(
                "Failed to open log file {:?}: {}. Falling back to os_log.",
                log_path, e
            );

            // Initialize fallback logging with `os_log` for macOS/iOS
            let oslogger_layer = OsLogger::new("net.nymtech.vpn.agent", "default");

            tracing_subscriber::registry()
                .compact()
                .with(oslogger_layer)
                .with(filter)
                .init();

            tracing::info!("Logger initialized with os_log due to file creation failure.");
        }
    }

    // Ensure logs are flushed immediately
    std::io::stdout().flush().unwrap();
    std::io::stderr().flush().unwrap();
}
