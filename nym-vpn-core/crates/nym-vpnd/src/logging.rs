// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, sync::Arc};

use sentry::integrations::tracing as sentry_tracing;
use tokio::{
    sync::{Mutex, mpsc},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::Level;
use tracing_appender::{non_blocking::WorkerGuard, rolling::RollingFileAppender};
#[cfg(target_os = "macos")]
use tracing_oslog::OsLogger;
use tracing_subscriber::{
    EnvFilter, Layer, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

use nym_vpn_lib_types::LogPath;

use crate::service;

static INFO_CRATES: &[&str; 14] = &[
    "hyper",
    "netlink_proto",
    "hickory_proto",
    "hickory_resolver",
    "hyper_util",
    "h2",
    "rustls",
    "surge_ping::client",
    "nym_statistics_common",
    "nym_sphinx_chunking",
    "nym_sphinx::preparer",
    "nym_task::manager",
    "nym_client_core::client::real_messages_control",
    "nym_client_core::client::received_buffer",
];

static WARN_CRATES: &[&str; 2] = &["hickory_server", "quinn::connection"];

pub struct Options {
    pub verbosity_level: Level,
    pub enable_file_log: bool,
    pub enable_stdout_log: bool,
    pub sentry: bool,
}

#[derive(Clone, Debug)]
pub struct FileAppender {
    inner: Arc<Mutex<Option<RollingFileAppender>>>,
    log_dir: PathBuf,
    log_file: String,
}

impl FileAppender {
    pub fn new() -> Self {
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

    pub async fn refresh(&mut self) {
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

pub struct LogFileRemover {
    command_rx: mpsc::UnboundedReceiver<()>,
    file_appender: FileAppender,
    shutdown_handle: CancellationToken,
}

impl LogFileRemover {
    pub fn spawn(
        file_appender: FileAppender,
        shutdown_handle: CancellationToken,
    ) -> (LogFileRemoverHandle, JoinHandle<()>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let file_remover = Self {
            command_rx: rx,
            file_appender,
            shutdown_handle,
        };
        let join_handle = tokio::spawn(file_remover.run());
        let remove_file_handle = LogFileRemoverHandle { tx };
        (remove_file_handle, join_handle)
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                Some(_) = self.command_rx.recv() => {
                    tracing::debug!("Received command to delete log file");
                    self.file_appender.refresh().await
                }
                _ = self.shutdown_handle.cancelled() => {
                    tracing::warn!("Exiting log file remover event loop");
                    break;
                }
            }
        }
    }
}

/// Interface for interacting with the log file remover.
#[derive(Clone)]
pub struct LogFileRemoverHandle {
    tx: mpsc::UnboundedSender<()>,
}

impl LogFileRemoverHandle {
    pub fn remove_log_file(&self) {
        if self.tx.send(()).is_err() {
            tracing::warn!("Log file remover channel is already closed");
        }
    }
}

pub struct LoggingSetup {
    pub worker_guard: WorkerGuard,
    pub file_appender: FileAppender,
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

pub struct LoggingSetupWithFileRemover {
    /// Handle for removing the log file
    pub log_file_remover_handle: LogFileRemoverHandle,
    /// Join handle for the file remover worker
    pub log_file_remover_join_handle: JoinHandle<()>,
    pub log_path: LogPath,
    /// A guard that flushes the log file when dropped.
    /// This worker guard should be retained for the lifetime of application.
    pub worker_guard: WorkerGuard,
}

pub fn default_log_path() -> LogPath {
    LogPath {
        dir: service::log_dir(),
        filename: service::DEFAULT_LOG_FILE.to_string(),
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
            format!("{crate_name}=info")
                .parse()
                .expect("failed to parse directive"),
        );
    }
    for crate_name in WARN_CRATES {
        env_filter = env_filter.add_directive(
            format!("{crate_name}=warn")
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
        // When debugging using WinDBG, the ANSI escape codes play havoc with the terminal output.
        let with_ansi = !(cfg!(debug_assertions) && cfg!(windows));

        let console_layer = tracing_subscriber::fmt::layer()
            .compact()
            .with_span_events(FmtSpan::CLOSE)
            .with_ansi(with_ansi);
        layers.push(console_layer.boxed());
    }

    if options.sentry {
        let layer = sentry_tracing::layer().event_filter(|md| match md.level() {
            &Level::ERROR | &Level::WARN => sentry_tracing::EventFilter::Event,
            &Level::TRACE => sentry_tracing::EventFilter::Ignore,
            _ => sentry_tracing::EventFilter::Breadcrumb,
        });
        layers.push(layer.boxed());
    }

    tracing_subscriber::registry()
        .with(layers)
        .with(env_filter)
        .init();

    log_panics::init();
    worker_guard
}

pub fn setup_logging_with_file_remover(
    options: Options,
    shutdown_token: CancellationToken,
) -> Option<LoggingSetupWithFileRemover> {
    let logging_setup = setup_logging(options);

    logging_setup.map(|logging_setup| {
        let (log_file_remover_handle, log_file_remover_join_handle) =
            LogFileRemover::spawn(logging_setup.file_appender, shutdown_token);

        LoggingSetupWithFileRemover {
            log_file_remover_handle,
            log_file_remover_join_handle,
            log_path: logging_setup.log_path,
            worker_guard: logging_setup.worker_guard,
        }
    })
}
