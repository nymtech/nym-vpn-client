// Copyright 2023-2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fs::OpenOptions, path::PathBuf, str::FromStr};

use sentry::integrations::tracing as sentry_tracing;
use tracing::Level;
use tracing_oslog::OsLogger;
use tracing_subscriber::{
    Layer, Registry, filter::LevelFilter, fmt::Layer as fmtLayer, layer::SubscriberExt,
    util::SubscriberInitExt,
};

pub fn init_logs(level: String, path: Option<PathBuf>, sentry: bool) -> bool {
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

    // Also ignore some of the more low level crates from the platform repo
    let filter = filter
        .add_directive("nym_client_core=info".parse().unwrap())
        .add_directive("nym_sphinx=info".parse().unwrap())
        .add_directive("nym_statistics_common=info".parse().unwrap());

    let registry = Registry::default().with(oslogger_layer);

    let mut layers = Vec::new();

    if let Some(path) = &path {
        // Ensure log directory exists
        if let Some(parent) = path.parent()
            && !parent.exists()
            && let Err(e) = std::fs::create_dir_all(parent)
        {
            eprintln!("Failed to create log directory {}: {e}", parent.display());
            return false;
        }

        // Attempting to get the tracing_appending solution to work was not successful.
        // Falling back to a more basic solution that does not support log rotation, for now.

        // Attempt to open the log file for writing
        let file = match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
        {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Failed to open log file {}: {e}", path.display());
                return false;
            }
        };

        let file_layer = fmtLayer::default()
            .with_writer(file)
            .with_ansi(false)
            .compact();

        layers.push(file_layer.boxed());
    }

    if sentry {
        let layer = sentry_tracing::layer().event_filter(|md| match md.level() {
            &Level::ERROR | &Level::WARN => sentry_tracing::EventFilter::Event,
            &Level::TRACE => sentry_tracing::EventFilter::Ignore,
            _ => sentry_tracing::EventFilter::Breadcrumb,
        });
        layers.push(layer.boxed());
    }

    match registry.with(layers).with(filter).try_init() {
        Ok(_) => {
            tracing::debug!("Logger initialized level: {level}, path?:{path:?}");
            true
        }
        Err(err) => {
            eprintln!("Failed to initialize logger: {err}");
            false
        }
    }
}
