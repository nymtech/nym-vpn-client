// Copyright 2023-2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fs::OpenOptions, path::PathBuf, str::FromStr};

use sentry::integrations::tracing as sentry_tracing;
use tracing::Level;
use tracing_subscriber::{
    Layer, Registry, filter::LevelFilter, fmt::Layer as fmtLayer, layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::error::VpnError;

pub fn init_logs(level: String, path: Option<PathBuf>, sentry: bool) -> Result<(), VpnError> {
    #[cfg(target_os = "ios")]
    let logger_layer = tracing_oslog::OsLogger::new("net.nymtech.vpn.agent", "default");
    #[cfg(target_os = "android")]
    let logger_layer = tracing_android::layer("libnymvpn").map_err(|err| VpnError::InitLogs {
        details: format!("Failed to create Android logger layer: {err}"),
    })?;

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

    let registry = Registry::default();

    #[cfg(any(target_os = "android", target_os = "ios"))]
    let registry = registry.with(logger_layer);

    let mut layers = Vec::new();

    if let Some(path) = &path {
        // Ensure log directory exists
        if let Some(parent) = path.parent()
            && let Err(e) = std::fs::create_dir_all(parent)
        {
            return Err(VpnError::InitLogs {
                details: format!("Failed to create log directory {}: {e}", parent.display()),
            });
        }

        // Attempting to get the tracing_appending solution to work was not successful.
        // Falling back to a more basic solution that does not support log rotation, for now.

        // Attempt to open the log file for writing
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|e| VpnError::InitLogs {
                details: format!("Failed to open log file {}: {e}", path.display()),
            })?;

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

    registry
        .with(layers)
        .with(filter)
        .try_init()
        .map_err(|err| VpnError::InitLogs {
            details: format!("Failed to initialize logger: {err}"),
        })?;

    tracing::info!("Setting log level: {level}, path?: {path:?}");

    Ok(())
}
