// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::str::FromStr;

use log::LevelFilter;
use oslog::OsLogger;


/// Path used for MacOS logs
#[cfg(target_os = "macos")]
const MACOS_LOG_FILEPATH: &str = "/var/log/nym-vpnd/daemon.log";

/// Environment variable name used for receiving ios log file path.
#[cfg(target_os = "ios")]
const IOS_LOG_FILEPATH_VAR: &str = "IOS_LOG_FILEPATH";

/// Enables and configures logging using the `log` and `oslog` libraries.
///
/// On call this subscriber attempts to parse filter level from the `"RUST_LOG"` environment variable.
/// If that is not set it defaults to `INFO` level.
///
/// As logs are not available to iOS or MacOS apps through the console, logs can be written to
/// file for handling. On iOS if a path is provided in the `"IOS__LOG_FILEPATH"` variable this
/// function will attempt to open that file and use it as the logging sink. On MacOS logs are
/// written to the static `"/var/log/nym-vpnd/daemon.log"`. If we are unable to open the
/// log filepath for either iOS or MacOS we default to writing to the default (console) output.
pub fn init_logs(level: String) {
    let log_builder = OsLogger::new("net.nymtech.vpn.agent")
        .level_filter(LevelFilter::from_str(&level).unwrap_or(LevelFilter::Info))
        .category_level_filter("hyper", LevelFilter::Warn)
        .category_level_filter("tokio_reactor", LevelFilter::Warn)
        .category_level_filter("reqwest", LevelFilter::Warn)
        .category_level_filter("mio", LevelFilter::Warn)
        .category_level_filter("want", LevelFilter::Warn)
        .category_level_filter("tungstenite", LevelFilter::Warn)
        .category_level_filter("tokio_tungstenite", LevelFilter::Warn)
        .category_level_filter("handlebars", LevelFilter::Warn)
        .category_level_filter("sled", LevelFilter::Warn);

    
    #[cfg(target_os = "macos")]
    if let Ok(f) = ::std::fs::File::create(MACOS_LOG_FILEPATH) {
        log_builder.target(env_logger::fmt::Target::Pipe(Box::new(f)));
    }

    #[cfg(target_os = "ios")]
    if let Ok(logfile_path) = ::std::env::var(IOS_LOG_FILEPATH_VAR) {
        if let Ok(f) = File::create(logfile_path) {
            log_builder.target(env_logger::fmt::Target::Pipe(Box::new(f)));
        }
    }

    match log_builder.init() {
        Ok(_) => {
            tracing::debug!("Logger initialized");
        }
        Err(e) => {
            tracing::error!("Failed to initialize os_log: {}", e);
        }
    };
}
