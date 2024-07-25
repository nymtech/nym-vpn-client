// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use log::LevelFilter;
use oslog::OsLogger;

pub fn init_logs() {
    OsLogger::new("net.nymtech.vpn.agent")
        .level_filter(LevelFilter::Debug)
        .category_level_filter("hyper", LevelFilter::Warn)
        .category_level_filter("tokio_reactor", LevelFilter::Warn)
        .category_level_filter("reqwest", LevelFilter::Warn)
        .category_level_filter("mio", LevelFilter::Warn)
        .category_level_filter("want", LevelFilter::Warn)
        .category_level_filter("tungstenite", LevelFilter::Warn)
        .category_level_filter("tokio_tungstenite", LevelFilter::Warn)
        .category_level_filter("handlebars", LevelFilter::Warn)
        .category_level_filter("sled", LevelFilter::Warn)
        .init()
        .expect("Could not init logs");
    tracing::debug!("Logger initialized");
}
