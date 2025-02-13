// Copyright 2023-2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fs::OpenOptions, path::PathBuf, str::FromStr};

use tracing_oslog::OsLogger;
use tracing_subscriber::{
    filter::LevelFilter, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Registry,
};

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

    let file_layer = path.as_ref().and_then(|path| {
        // Ensure log directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!("Failed to create log directory {}: {e}", parent.display());
                }
            }
        }

        // Attempting to get the tracing_appending solution to work was not successful.
        // Falling back to a more basic solution that does not support log rotation, for now.

        // Attempt to open the log file for writing
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .ok()
            .map(|file| {
                Layer::default()
                    .with_writer(file)
                    .with_ansi(false)
                    .compact()
            })
    });

    let result = if let Some(file_layer) = file_layer {
        registry.with(file_layer).with(filter).try_init()
    } else {
        registry.with(filter).try_init()
    };

    if let Err(err) = result {
        eprintln!("Failed to initialize logger: {err}");
    } else {
        tracing::debug!("Logger initialized level: {level}, path?:{path:?}");
    }
}
