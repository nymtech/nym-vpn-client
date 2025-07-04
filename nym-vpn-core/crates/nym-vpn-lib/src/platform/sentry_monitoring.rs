// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use sentry::ClientInitGuard;
use std::time::Duration;

pub fn init() -> Option<ClientInitGuard> {
    let dsn = std::env::var("VPNLIB_SENTRY_DSN")
        .ok()
        .or_else(|| option_env!("VPNLIB_SENTRY_DSN").map(|s| s.to_string()))?;

    println!("⚠ sentry monitoring enabled ⚠");
    let guard = sentry::init((
        dsn,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            send_default_pii: false,
            sample_rate: 1.0,
            traces_sample_rate: 1.0,
            enable_logs: true,
            shutdown_timeout: Duration::from_secs(2),
            ..Default::default()
        },
    ));
    Some(guard)
}
