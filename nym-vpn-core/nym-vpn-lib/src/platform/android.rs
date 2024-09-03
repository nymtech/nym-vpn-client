// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::mobile::tunnel_settings::TunnelNetworkSettings;
use log::LevelFilter;
use std::fmt::Debug;
use std::os::fd::RawFd;

pub(crate) fn init_logs(level: String) {
    use android_logger::{Config, FilterBuilder};
    let levels = level + ",tungstenite=warn,mio=warn,tokio_tungstenite=warn";

    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Trace)
            .with_tag("libnymvpn")
            .with_filter(FilterBuilder::new().parse(levels.as_str()).build()),
    );
    log::debug!("Logger initialized");
}

#[uniffi::export(with_foreign)]
pub trait AndroidTunProvider: Send + Sync + Debug {
    fn bypass(&self, socket: i32);
    fn configure_tunnel(
        &self,
        config: TunnelNetworkSettings,
    ) -> Result<RawFd, crate::platform::error::FFIError>;
}
