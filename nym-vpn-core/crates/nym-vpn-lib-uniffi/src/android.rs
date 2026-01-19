// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::str::FromStr;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{Registry, layer::SubscriberExt, util::SubscriberInitExt};

use crate::error::VpnError;

pub(crate) fn init_logs(level: String) -> Result<(), VpnError> {
    let logcat_layer = tracing_android::layer("libnymvpn").unwrap();

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

    let registry = Registry::default().with(logcat_layer);

    registry
        .with(filter)
        .try_init()
        .map_err(|err| VpnError::CreateLogFile {
            details: format!("Failed to initialize logger: {err}"),
        })
}
