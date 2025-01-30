// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tracing_appender::non_blocking::WorkerGuard;
#[cfg(target_os = "macos")]
use tracing_oslog::OsLogger;
use tracing_subscriber::{
    filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

use crate::service;

pub struct Options {
    pub enable_file_log: bool,
    pub enable_stdout_log: bool,
}

pub fn setup_logging(options: Options) -> Option<WorkerGuard> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        .add_directive("hyper::proto=info".parse().unwrap())
        .add_directive("netlink_proto=info".parse().unwrap());

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
            .with_writer(file_writer)
            .with_ansi(false);
        layers.push(file_layer.boxed());
        Some(worker_guard)
    } else {
        None
    };

    if options.enable_stdout_log {
        let console_layer = tracing_subscriber::fmt::layer().compact().with_ansi(true);
        layers.push(console_layer.boxed());
    }

    tracing_subscriber::registry()
        .with(layers)
        .with(env_filter)
        .init();

    log_panics::init();

    worker_guard
}
