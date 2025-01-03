// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use log::LevelFilter;

pub(crate) fn init_logs(level: String) {
    use android_logger::{Config, FilterBuilder};
    let levels = level + ",tungstenite=warn,mio=warn,tokio_tungstenite=warn";

    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Trace)
            .with_tag("libnymvpn")
            .with_filter(FilterBuilder::new().parse(levels.as_str()).build()),
    );
    tracing::debug!("Logger initialized");
}
