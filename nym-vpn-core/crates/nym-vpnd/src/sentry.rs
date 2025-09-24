// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use sentry::{ClientInitGuard, Level};
use std::{borrow::Cow, sync::Arc, time::Duration};

use crate::{config::GlobalConfig, environment};

static EXCLUDED_ERRORS: [&str; 6] = [
    "offline",
    "client is not authenticated",
    "connection reset",
    "connection refused",
    "connection closed",
    "connection timed out",
];

pub async fn init_sentry() -> Option<ClientInitGuard> {
    if !GlobalConfig::sentry_enabled().await {
        return None;
    }

    let Some(dsn) = environment::sentry_dsn() else {
        eprintln!("failed to init sentry: SENTRY_DSN is not set");
        return None;
    };

    let os_info = nym_platform_metadata::SysInfo::new();

    println!("Sentry monitoring enabled");
    let guard = sentry::init((
        dsn,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            send_default_pii: false,
            sample_rate: 1.0,
            traces_sample_rate: 1.0,
            enable_logs: true,
            shutdown_timeout: Duration::from_secs(2),
            server_name: Some(Cow::Borrowed("nym")),
            before_send: Some(Arc::new(|mut event| {
                if matches!(event.level, Level::Error | Level::Warning)
                    && let Some(message) = &event.message
                    && EXCLUDED_ERRORS
                        .iter()
                        .any(|err| message.to_lowercase().contains(err))
                {
                    event.level = Level::Debug; // Change level to Debug
                }
                Some(event)
            })),
            ..Default::default()
        },
    ));
    sentry::configure_scope(|scope| {
        scope.set_tag("os_version", &os_info.os_version);
        if !os_info.extra.is_empty() {
            scope.set_tag("extra_metadata", os_info.extra.join(", "));
        }
        scope.set_user(Some(sentry::User {
            id: Some(os_info.hash_identifier()), // anonymized user identifier
            ip_address: None,
            ..Default::default()
        }));
    });

    Some(guard)
}
