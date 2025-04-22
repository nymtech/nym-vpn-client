// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::Level;
use tracing_appender::{non_blocking::WorkerGuard, rolling::RollingFileAppender};
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

static INFO_CRATES: &[&str; 12] = &[
    "hyper",
    "netlink_proto",
    "hickory_proto",
    "hickory_resolver",
    "hyper_util",
    "h2",
    "rustls",
    "nym_statistics_common",
    "nym_client_core",
    "nym_sphinx_chunking",
    "nym_sphinx::preparer",
    "nym_task::manager",
];

static WARN_CRATES: &[&str; 1] = &["hickory_server"];

#[derive(Clone, Debug)]
pub(crate) struct FileAppender {
    inner: Arc<Mutex<Option<RollingFileAppender>>>,
    log_dir: PathBuf,
    log_file: String,
}

impl FileAppender {
    pub(crate) fn new() -> Self {
        let log_dir = service::log_dir();
        let log_file = service::DEFAULT_LOG_FILE.to_string();

        let mut log_file_path = log_dir.clone();
        log_file_path.push(&log_file);
        let mut old_log_file_path = log_dir.clone();
        old_log_file_path.push(service::DEFAULT_OLD_LOG_FILE);

        if std::fs::exists(&log_file_path).unwrap_or(false)
            && std::fs::rename(&log_file_path, &old_log_file_path).is_err()
        {
            tracing::warn!(
                "Log rotation could not be performed, we're going to just append to the same file"
            );
        }

        let inner = Arc::new(Mutex::new(Some(tracing_appender::rolling::never(
            log_dir.clone(),
            &log_file,
        ))));
        Self {
            inner,
            log_dir,
            log_file,
        }
    }

    pub(crate) async fn refresh(&mut self) {
        let mut file_path = self.log_dir.clone();
        file_path.push(&self.log_file);
        let mut file_lock = self.inner.lock().await;
        // drop the file appeneder, so that we can remove the file in the next step
        let _ = file_lock.take();
        if let Err(err) = tokio::fs::remove_file(file_path).await {
            tracing::warn!("Could not remove log file: {err}");
            return;
        }
        // re-create the empty file
        *file_lock = Some(tracing_appender::rolling::never(
            &self.log_dir,
            &self.log_file,
        ));
    }
}

pub(crate) struct LogFileRemover {
    tunnel_event_rx: mpsc::Receiver<()>,
    logging_setup: LoggingSetup,
    shutdown_token: CancellationToken,
}

impl LogFileRemover {
    pub(crate) fn new(
        tunnel_event_rx: mpsc::Receiver<()>,
        logging_setup: LoggingSetup,
        shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            tunnel_event_rx,
            logging_setup,
            shutdown_token,
        }
    }

    pub(crate) async fn run(mut self) -> WorkerGuard {
        loop {
            tokio::select! {
                Some(_) = self.tunnel_event_rx.recv() => {
                    tracing::debug!("Received command to delete log file");
                    self.handle_delete_log_file().await;
                }
                _ = self.shutdown_token.cancelled() => {
                    tracing::debug!("Received shutdown signal");
                    break;
                }
                else => {
                    tracing::warn!("Event loop is interrupted");
                    break;
                }
            }
        }
        self.logging_setup.worker_guard
    }

    pub(crate) async fn handle_delete_log_file(&mut self) {
        self.logging_setup.file_appender.refresh().await
    }
}

pub struct LoggingSetup {
    worker_guard: WorkerGuard,
    file_appender: FileAppender,
    pub log_path: LogPath,
}

impl LoggingSetup {
    pub fn new(worker_guard: WorkerGuard, file_appender: FileAppender) -> Self {
        let log_path = LogPath::new(
            file_appender.log_dir.clone(),
            file_appender.log_file.to_string(),
        );
        Self {
            worker_guard,
            file_appender,
            log_path,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LogPath {
    pub dir: PathBuf,
    pub filename: String,
}

impl LogPath {
    pub fn new(log_dir: PathBuf, log_file: String) -> Self {
        Self {
            dir: log_dir,
            filename: log_file,
        }
    }
}

impl Default for LogPath {
    fn default() -> Self {
        Self {
            dir: service::log_dir(),
            filename: service::DEFAULT_LOG_FILE.to_string(),
        }
    }
}
struct FileManager {
    file_appender: FileAppender,
}

impl FileManager {
    pub fn new(file_appender: FileAppender) -> Self {
        Self { file_appender }
    }
}

impl std::io::Write for FileManager {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(self
            .file_appender
            .inner
            .blocking_lock()
            .as_mut()
            .map(|writer| writer.write(buf))
            .transpose()?
            .unwrap_or(0))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file_appender
            .inner
            .blocking_lock()
            .as_mut()
            .map(|writer| writer.flush())
            .transpose()?;
        Ok(())
    }
}

pub fn setup_logging(options: Options) -> Option<LoggingSetup> {
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
        let file_appender = FileAppender::new();
        let file_manager = FileManager::new(file_appender.clone());
        let (file_writer, worker_guard) = tracing_appender::non_blocking(file_manager);
        let file_layer = tracing_subscriber::fmt::layer()
            .compact()
            .with_span_events(FmtSpan::CLOSE)
            .with_writer(file_writer)
            .with_ansi(false);
        layers.push(file_layer.boxed());
        Some(LoggingSetup::new(worker_guard, file_appender))
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
