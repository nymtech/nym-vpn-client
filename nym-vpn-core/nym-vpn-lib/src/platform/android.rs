// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Debug;
use super::*;
use std::io;
use std::os::fd::RawFd;

pub fn init_logs(level: String) {
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
    fn configure_wg(&self, config: WgConfig) -> Result<(), crate::platform::error::FFIError>;
    fn configure_nym(&self, config: NymConfig) -> Result<RawFd, crate::platform::error::FFIError>;
}
