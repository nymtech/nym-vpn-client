// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
#[cfg(target_os = "macos")]
use tracing_oslog::OsLogger;
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

use crate::service;

pub struct Options {
    pub verbosity_level: Level,
    pub enable_file_log: bool,
    pub enable_stdout_log: bool,
}

static INFO_CRATES: &[&str; 16] = &[
    "hyper",
    "netlink_proto",
    "hickory_proto",
    "hickory_resolver",
    "hyper_util",
    "h2",
    "rustls",
    "nym_statistics_common",
    "nym_client_core",
    "nym_vpn_account_controller",
    "nym_vpn_store",
    "nym_vpn_api_client::jwt",
    "nym_sphinx_chunking",
    "nym_sphinx::preparer",
    "nym_authenticator_client",
    "nym_task::manager",
];

static WARN_CRATES: &[&str; 1] = &["hickory_server"];

pub fn setup_logging(options: Options) -> Option<WorkerGuard> {
    let mut env_filter = EnvFilter::builder()
        .with_default_directive(options.verbosity_level.into())
        .from_env_lossy();

    for crate_name in INFO_CRATES {
        env_filter = env_filter.add_directive(
            format!("{}=info", crate_name)
                .parse()
                .expect("failed to parse directive"),
        );
    }
    for crate_name in WARN_CRATES {
        env_filter = env_filter.add_directive(
            format!("{}=warn", crate_name)
                .parse()
                .expect("failed to parse directive"),
        );
    }

    let mut layers = Vec::new();

    // Create oslog output on macOS for debugging purposes
    #[cfg(target_os = "macos")]
    layers.push(OsLogger::new("net.nymtech.vpn.agent", "default").boxed());

    // Create file logger but only when running as a service on windows or macos
    let worker_guard = if options.enable_file_log {
        let log_dir = service::log_dir();
        let file_appender = tracing_appender::rolling::never(log_dir, service::DEFAULT_LOG_FILE);
        let (file_writer, worker_guard) = tracing_appender::non_blocking(file_appender);
        let file_layer = tracing_subscriber::fmt::layer()
            .compact()
            .with_span_events(FmtSpan::CLOSE)
            .with_writer(file_writer)
            .with_ansi(false);
        layers.push(file_layer.boxed());
        Some(worker_guard)
    } else {
        None
    };

    if options.enable_stdout_log {
        let console_layer = tracing_subscriber::fmt::layer()
            .compact()
            .with_span_events(FmtSpan::CLOSE)
            .with_ansi(true);
        layers.push(console_layer.boxed());
    }

    tracing_subscriber::registry()
        .with(layers)
        .with(env_filter)
        .init();

    log_panics::init();

    let build_info = nym_bin_common::bin_info_local_vergen!();
    tracing::info!(
        "{} {} ({})",
        build_info.binary_name,
        build_info.build_version,
        build_info.commit_sha
    );
    worker_guard
}
