// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use sentry::ClientInitGuard;
use sha2::{Digest, Sha256};
use std::{
    borrow::Cow,
    collections::HashSet,
    sync::{Arc, OnceLock},
    time::Duration,
};

use crate::{config::GlobalConfigFile, environment};

static EXCLUDED_ERRORS: OnceLock<HashSet<&'static str>> = OnceLock::new();

fn get_excluded_errors() -> &'static HashSet<&'static str> {
    EXCLUDED_ERRORS.get_or_init(|| {
        HashSet::from([
            "offline",
            "client is not authenticated",
            "connection reset",
            "connection refused",
            "connection closed",
            "connection timed out",
        ])
    })
}

pub fn init_sentry() -> Option<ClientInitGuard> {
    if !GlobalConfigFile::sentry_enabled() {
        return None;
    }

    let Some(dsn) = environment::sentry_dsn() else {
        eprintln!("failed to init sentry: SENTRY_DSN is not set");
        return None;
    };

    let os_info = nym_vpn_lib::SysInfo::new();

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
            before_send_log: Some(Arc::new(|log| {
                if get_excluded_errors()
                    .iter()
                    .any(|err| log.body.contains(err.to_lowercase().as_str()))
                {
                    tracing::info!("Excluded log: {}", log.body); // Keep excluded logs in breadcrumbs
                    return None; // Exclude this log
                }
                Some(log)
            })),
            ..Default::default()
        },
    ));
    sentry::configure_scope(|scope| {
        scope.set_tag("os_version", &os_info.os_version);
        scope.set_tag("extra_metadata", os_info.extra.join(", "));
        scope.set_user(Some(sentry::User {
            id: Some(anonymize_identifier(&os_info)), // anonymized user identifier
            ip_address: None,
            ..Default::default()
        }));
    });

    Some(guard)
}

fn anonymize_identifier(os_info: &nym_vpn_lib::SysInfo) -> String {
    let identifier = format!(
        "{} {} {} {}",
        os_info.os_version,
        os_info.arch,
        os_info.extra.join(" "),
        sysinfo::System::host_name().unwrap_or_else(|| "unknown".to_string())
    );
    let hash = Sha256::digest(identifier.as_bytes());
    format!("{hash:x}")
}
