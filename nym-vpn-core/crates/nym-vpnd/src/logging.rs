// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only
#[cfg(target_os = "macos")]
use std::env;

use tracing_appender::non_blocking::WorkerGuard;

use crate::service;

pub fn setup_logging(_as_service: bool) {
    #[cfg(target_os = "macos")]
    if _as_service {
        let log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
        let log_path = service::log_dir().with_file_name(service::DEFAULT_LOG_FILE);
        nym_vpn_lib::swift::init_logs(log_level, Some(log_path));
        return;
    }

    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env()
        .unwrap()
        .add_directive("hyper::proto=info".parse().unwrap())
        .add_directive("netlink_proto=info".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();
}

#[allow(unused)]
pub fn setup_logging_to_file() -> WorkerGuard {
    let log_dir = service::log_dir();

    println!("log_dir: {}", log_dir.display());

    let file_appender = tracing_appender::rolling::never(log_dir, service::DEFAULT_LOG_FILE);
    let (file_writer, worker_guard) = tracing_appender::non_blocking(file_appender);

    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env()
        .unwrap()
        .add_directive("hyper::proto=info".parse().unwrap())
        .add_directive("netlink_proto=info".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .with_writer(file_writer)
        .with_ansi(false)
        .init();

    std::panic::set_hook(Box::new(|panic| {
        tracing::error!(message = %panic);
    }));

    worker_guard
}
