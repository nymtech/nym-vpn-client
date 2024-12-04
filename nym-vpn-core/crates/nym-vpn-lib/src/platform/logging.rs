// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tracing_subscriber::{filter::LevelFilter, EnvFilter};

pub(crate) const DEFAULT_LOG_FILE: &str = "nym-vpn-lib.log";

/// Enables and configures logging using the `tracing-subscriber` and `tracing-appender` libraries. If a non-empty
/// string is provided for the path, we attempt to cannonicalize and parse the path such that logs are written to file.
/// If the provided path is a directory, logs will be written to "{provided_dir}/nym-vpn-lib.log".
///
/// On call this subscriber attempts to parse filter level from the (default) logging environment variable `"RUST_LOG"`.
/// If that is not set it defaults to `INFO` level.
///
/// Android logging is handled using the [`android_logger`] crate.
//
// Is there a state object  associated with `uniffi::export` stored in `nym-vpn-lib`? As is, the
// [`tracing_appender::rolling::RollingFileAppender`] may write in blocking mode. Without somewhere to store the
// worker guard provided by the construction of a [`tracing_appender::non_blocking::NonBlocking`] the appender
// may not flush properly on drop (i.e. around a crash).
//
// let (writer, worker_guard) = tracing_appender::non_blocking(appender);
//
// see https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/struct.WorkerGuard.html
pub fn init_logger(path_str: &str) {
    #[cfg(target_os = "android")]
    {
        init_logs(log_level);
        return;
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    let mut filter = EnvFilter::builder()
        .with_env_var("RUST_LOG")
        .with_default_directive(LevelFilter::INFO)
        .from_env_lossy();

    #[cfg(not(any(target_os = "ios", target_os = "macos")))]
    let mut filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        .add_directive("netlink_proto=info".parse().unwrap());

    filter = filter
        .add_directive("hyper=info".parse().unwrap())
        .add_directive("tokio_reactor=warn".parse().unwrap())
        .add_directive("reqwest=warn".parse().unwrap())
        .add_directive("mio=warn".parse().unwrap())
        .add_directive("want=warn".parse().unwrap())
        .add_directive("tungstenite=warn".parse().unwrap())
        .add_directive("tokio_tungstenite=warn".parse().unwrap())
        .add_directive("handlebars=warn".parse().unwrap())
        .add_directive("sled=warn".parse().unwrap());

    let log_builder = tracing_subscriber::fmt().with_env_filter(filter).compact();

    if let Some(appender) = try_make_writer(path_str) {
        std::panic::set_hook(Box::new(|panic| {
            tracing::error!(message = %panic);
        }));

        log_builder.with_writer(appender).init();
    } else {
        log_builder.init();
    }
}

fn try_make_writer(path_str: &str) -> Option<tracing_appender::rolling::RollingFileAppender> {
    if path_str.is_empty() {
        return None;
    }

    let path = ::std::path::Path::new(path_str).canonicalize().ok()?;

    let (maybe_log_dir, filename) = if path.is_dir() {
        (
            Some(path.as_path()),
            ::std::path::Path::new(DEFAULT_LOG_FILE),
        )
    } else if path.is_file() {
        (
            path.parent(),
            ::std::path::Path::new(path.file_name().unwrap()),
        )
    } else {
        return None;
    };

    // make sure that the path provides a directory, the directory exists and we have permission to access it.
    if !maybe_log_dir.is_some_and(|d| d.try_exists().is_ok_and(|exists| exists)) {
        return None;
    };

    let log_dir = maybe_log_dir.unwrap();

    println!("log_path: {}", path.display());

    Some(tracing_appender::rolling::never(log_dir, filename))
}

#[cfg(target_os = "android")]
pub(crate) fn init_logs(level: String) {
    use android_logger::{Config, FilterBuilder};
    let levels = level + ",tungstenite=warn,mio=warn,tokio_tungstenite=warn";

    android_logger::init_once(
        Config::default()
            .with_max_level(log::LevelFilter::Trace)
            .with_tag("libnymvpn")
            .with_filter(FilterBuilder::new().parse(levels.as_str()).build()),
    );
    log::debug!("Logger initialized");
}
