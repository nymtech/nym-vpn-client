// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, str::FromStr};

use tracing_oslog::OsLogger;
use tracing_subscriber::{
    filter::LevelFilter, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Registry,
};

pub(crate) const DEFAULT_LOG_FILE: &str = "nym-vpn-lib.log";

pub fn init_logs(level: String, path: Option<PathBuf>) {
    let oslogger_layer = OsLogger::new("net.nymtech.vpn.agent", "default");

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

    let registry = Registry::default().with(oslogger_layer);

    let result = match path {
        Some(ref log_path) => {
            // If a path was provided attempt to add a tracing_subscriber Layer that logs  to file
            match try_make_writer(log_path.clone()) {
                Some(appender) => {
                    let (non_blocking, _) = tracing_appender::non_blocking(appender);
                    let file_appender_layer = Layer::default().with_writer(non_blocking);
                    registry.with(file_appender_layer).with(filter).try_init()
                }
                None => {
                    // if we failed to make the file appender init just the os_log version.
                    tracing::error!("Failed to initialize file logger: bad path: \"{log_path:?}\"");
                    registry.with(filter).try_init()
                }
            }
        }
        // no file path provided -- init os_log logger
        None => registry.with(filter).try_init(),
    };

    match result {
        Ok(_) => {
            tracing::debug!("Logger initialized level: {level}, path?:{path:?}");
        }
        Err(e) => {
            tracing::error!("Failed to initialize os_log: {}", e);
        }
    };
}

fn try_make_writer(path: PathBuf) -> Option<tracing_appender::rolling::RollingFileAppender> {
    let path = path.canonicalize().ok()?;

    let (maybe_log_dir, filename) = if path.is_dir() {
        (
            Some(path.as_path()),
            ::std::path::Path::new(DEFAULT_LOG_FILE),
        )
    } else if path.is_file() {
        (
            path.parent(),
            ::std::path::Path::new(path.file_name().unwrap()),
        )
    } else {
        return None;
    };

    // make sure that the path provides a directory, the directory exists and we have permission to access it.
    if !maybe_log_dir.is_some_and(|d| d.try_exists().is_ok_and(|exists| exists)) {
        return None;
    };

    let log_dir = maybe_log_dir.unwrap();

    Some(tracing_appender::rolling::never(log_dir, filename))
}
