// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use itertools::Itertools;
use nym_common::trace_err_chain;
use opentelemetry::trace::{TraceContextExt, TracerProvider};
use sentry::integrations::tracing as sentry_tracing;
use tokio::{
    sync::{Mutex, mpsc},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::{Dispatch, Event, Level, Subscriber, dispatcher::WeakDispatch};
use tracing_appender::{non_blocking::WorkerGuard, rolling::RollingFileAppender};
use tracing_opentelemetry::get_otel_context;
use tracing_subscriber::{
    EnvFilter, Layer,
    fmt::{FmtContext, FormatEvent, FormatFields, format::FmtSpan},
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
};

use nym_vpn_lib_types::LogPath;

#[cfg(any(target_os = "android", target_os = "ios"))]
pub const DEFAULT_LOG_FILE: &str = "libnymvpn.log";

#[cfg(any(target_os = "android", target_os = "ios"))]
pub const DEFAULT_OLD_LOG_FILE: &str = "libnymvpn.log";

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub const DEFAULT_LOG_FILE: &str = "nym-vpnd.log";

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub const DEFAULT_OLD_LOG_FILE: &str = "nym-vpnd.old.log";

static INFO_TARGETS: [&str; 16] = [
    "hyper",
    "netlink_proto",
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
    "tonic::transport::server",
    "hickory_resolver",
    "hickory_proto",
    "hickory_net",
];

static WARN_TARGETS: [&str; 3] = ["hickory_server", "quinn::connection", "zbus"];

pub struct Options {
    pub verbosity_level: Level,
    pub enable_stdout_log: bool,
    pub enable_json_log: bool,
    pub log_dir: Option<PathBuf>,
    pub sentry: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum SharedFileAppenderError {
    #[error("failed to init rolling appender")]
    InitRollingAppender(#[from] tracing_appender::rolling::InitError),

    #[error("I/O error")]
    Io(#[from] std::io::Error),
}

#[derive(Clone, Debug)]
pub struct SharedFileAppender {
    inner: Arc<Mutex<Option<RollingFileAppender>>>,
    log_dir: PathBuf,
    log_file_name: String,
}

impl SharedFileAppender {
    /// Create new file appender and making a backup of existing log file
    ///
    /// ## Arguments
    ///
    /// * `log_dir`: Directory where the log files are stored.
    /// * `log_file_name`: Current log file (i.e. "nym_vpn.log")
    /// * `old_log_file_name`: Backup log file (i.e. "nym_vpn.log.old")
    pub fn new(
        log_dir: PathBuf,
        log_file_name: &str,
        old_log_file_name: &str,
    ) -> Result<Self, SharedFileAppenderError> {
        let log_file_path = log_dir.join(log_file_name);
        let old_log_file_path = log_dir.join(old_log_file_name);

        if let Err(err) = std::fs::rename(&log_file_path, &old_log_file_path)
            && err.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(
                "Log rotation could not be performed, we're going to just append to the same file"
            );
        }

        let rolling_appender = Self::create_rolling_appender(&log_dir, log_file_name)?;

        Ok(Self {
            inner: Arc::new(Mutex::new(Some(rolling_appender))),
            log_dir,
            log_file_name: log_file_name.to_owned(),
        })
    }

    /// Empty the log file by removing it and opening a new one in its place.
    pub async fn delete_log_file(&mut self) -> Result<(), SharedFileAppenderError> {
        let file_path = self.log_dir.join(&self.log_file_name);

        let mut file_lock = self.inner.lock().await;
        // drop the file appeneder, so that we can remove the file in the next step
        let _ = file_lock.take();

        tokio::fs::remove_file(file_path).await.or_else(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(err)
            }
        })?;

        let rolling_appender = Self::create_rolling_appender(&self.log_dir, &self.log_file_name)
            .map_err(SharedFileAppenderError::InitRollingAppender)?;

        *file_lock = Some(rolling_appender);

        Ok(())
    }

    pub fn create_rolling_appender(
        log_dir: &Path,
        log_file_name: &str,
    ) -> Result<tracing_appender::rolling::RollingFileAppender, tracing_appender::rolling::InitError>
    {
        tracing_appender::rolling::RollingFileAppender::builder()
            .rotation(tracing_appender::rolling::Rotation::NEVER)
            .filename_prefix(log_file_name)
            .build(log_dir)
    }
}

impl std::io::Write for SharedFileAppender {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.inner.blocking_lock().as_mut() {
            Some(writer) => writer.write(buf),
            None => Ok(buf.len()),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner
            .blocking_lock()
            .as_mut()
            .map(|writer| writer.flush())
            .transpose()?;
        Ok(())
    }
}

pub struct LogFileRemover {
    command_rx: mpsc::UnboundedReceiver<()>,
    file_appender: SharedFileAppender,
    shutdown_handle: CancellationToken,
}

impl LogFileRemover {
    pub fn spawn(
        file_appender: SharedFileAppender,
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
                    if let Err(err) = self.file_appender.delete_log_file().await {
                        trace_err_chain!(err, "failed to delete log file");
                    }
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
    pub file_appender: SharedFileAppender,
    pub log_path: LogPath,
}

impl LoggingSetup {
    pub fn new(worker_guard: WorkerGuard, file_appender: SharedFileAppender) -> Self {
        let log_path = LogPath::new(
            file_appender.log_dir.clone(),
            file_appender.log_file_name.clone(),
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

/// Layer which sole purpose is to capture `Dispatch`
struct JsonLogLayer {
    dispatch: Arc<OnceLock<WeakDispatch>>,
}

impl JsonLogLayer {
    pub fn new(dispatch: Arc<OnceLock<WeakDispatch>>) -> Self {
        Self { dispatch }
    }
}

impl<S> Layer<S> for JsonLogLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_register_dispatch(&self, dispatch: &Dispatch) {
        self.dispatch.set(dispatch.downgrade()).ok();
    }
}

struct JsonLogFormatter {
    enable_opentelemetry: bool,
    dispatch: Arc<OnceLock<WeakDispatch>>,
}

impl JsonLogFormatter {
    pub fn new(enable_opentelemetry: bool, dispatch: Arc<OnceLock<WeakDispatch>>) -> Self {
        Self {
            enable_opentelemetry,
            dispatch,
        }
    }
}

impl<S, N> FormatEvent<S, N> for JsonLogFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        write!(writer, "{{")?;
        if self.enable_opentelemetry
            && let Some(dispatch) = self.dispatch.get().and_then(|d| d.upgrade())
            && let Some((trace_id, span_id)) = ctx.event_scope().and_then(|mut scope| {
                scope.find_map(|span_ref| {
                    let otel = get_otel_context(&span_ref.id(), &dispatch)?;
                    let span = otel.span();
                    let span_ctx = span.span_context();

                    let trace_id = span_ctx.trace_id();
                    let span_id = span_ctx.span_id();

                    Some((trace_id.to_string(), span_id.to_string()))
                })
            })
        {
            write!(writer, r#""trace_id":"{trace_id}","span_id":"{span_id}","#)?;
        }
        write!(
            writer,
            r#""timestamp":"{}","level":"{}","target":"{}","#,
            time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_else(|_| "-".into()),
            event.metadata().level(),
            event.metadata().target(),
        )?;
        write!(writer, r#""fields":"#)?;
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        write!(writer, "}}")?;

        writeln!(writer)
    }
}

/// Install the global tracing subscriber and panic logger.
///
/// Log filtering is taken from `RUST_LOG` when set, otherwise from `options.verbosity_level`
/// combined with built-in per-crate overrides. Output goes to the platform logger (os_log on
/// Apple platforms, logcat on Android) and, depending on `options`, to stdout, a log file in
/// `options.log_dir` (the previous log file is kept as a backup) and Sentry.
///
/// Returns `None` when `options.log_dir` is `None` or the log file cannot be opened, in which case
/// the error is logged once the subscriber is installed and file logging is disabled. Otherwise
/// the returned [`LoggingSetup`] must be kept alive for as long as logs should be written to the
/// file.
///
/// ## Panics
///
/// Panics if a global tracing subscriber has already been installed.
pub fn setup_logging(options: Options) -> Option<LoggingSetup> {
    // Right now we only use opentelemetry for generating trace ID and span ID in JSON logs,
    // which are harder to read but better for automated tools.
    // ! This does not configure any additional telemetry, it's just additional data added locally !
    let enable_opentelemetry = options.enable_json_log;

    // Setup from RUST_LOG if set and not empty. Otherwise use production configuration
    let env_filter = if std::env::var(EnvFilter::DEFAULT_ENV).is_ok_and(|s| !s.trim().is_empty()) {
        EnvFilter::builder()
            .with_default_directive(options.verbosity_level.into())
            .from_env_lossy()
    } else {
        let default_directives = std::iter::once(options.verbosity_level.to_string())
            .chain(INFO_TARGETS.iter().map(|c| format!("{c}=info")))
            .chain(WARN_TARGETS.iter().map(|c| format!("{c}=warn")))
            .join(",");
        EnvFilter::new(default_directives)
    };

    // Platform log layers
    #[cfg(target_os = "android")]
    let android_layer =
        Some(tracing_android::layer("libnymvpn").expect("tag contains nul terminator"));
    #[cfg(not(target_os = "android"))]
    let android_layer: Option<tracing_subscriber::fmt::Layer<_>> = None;

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    let os_logger = Some(tracing_oslog::OsLogger::new(
        "net.nymtech.vpn.agent",
        "default",
    ));
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    let os_logger: Option<tracing_subscriber::fmt::Layer<_>> = None;

    // Dispatch proxy layer
    let dispatch = Arc::new(OnceLock::new());
    let json_layer = JsonLogLayer::new(dispatch.clone());

    // File log setup
    let (file_appender, file_appender_error) = match options
        .log_dir
        .map(|log_dir| SharedFileAppender::new(log_dir, DEFAULT_LOG_FILE, DEFAULT_OLD_LOG_FILE))
        .transpose()
    {
        Ok(file_appender) => (file_appender, None),
        Err(err) => (None, Some(err)),
    };
    let (mut file_writer, logging_setup) = if let Some(file_appender) = file_appender {
        let (file_writer, worker_guard) = tracing_appender::non_blocking(file_appender.clone());

        (
            Some(file_writer),
            Some(LoggingSetup::new(worker_guard, file_appender)),
        )
    } else {
        (None, None)
    };

    let file_layer_json = if options.enable_json_log
        && let Some(file_writer) = file_writer.take()
    {
        Some(
            tracing_subscriber::fmt::layer()
                .with_span_events(FmtSpan::CLOSE)
                .with_writer(file_writer)
                .with_ansi(false)
                .json()
                .event_format(JsonLogFormatter::new(
                    enable_opentelemetry,
                    dispatch.clone(),
                )),
        )
    } else {
        None
    };

    let file_layer_plain = if !options.enable_json_log
        && let Some(file_writer) = file_writer.take()
    {
        Some(
            tracing_subscriber::fmt::layer()
                .with_span_events(FmtSpan::CLOSE)
                .with_writer(file_writer)
                .with_ansi(false),
        )
    } else {
        None
    };

    // Console log setup
    let console_layer_plain = if options.enable_stdout_log && !options.enable_json_log {
        Some(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE))
    } else {
        None
    };

    let console_layer_json = if options.enable_stdout_log && options.enable_json_log {
        let console_layer = tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE);
        let formatted_console = console_layer.json().event_format(JsonLogFormatter::new(
            enable_opentelemetry,
            dispatch.clone(),
        ));
        Some(formatted_console)
    } else {
        None
    };

    // Sentry
    let sentry_layer = if options.sentry {
        Some(sentry_tracing::layer().event_filter(|md| match md.level() {
            &Level::ERROR | &Level::WARN => sentry_tracing::EventFilter::Event,
            &Level::TRACE => sentry_tracing::EventFilter::Ignore,
            _ => sentry_tracing::EventFilter::Breadcrumb,
        }))
    } else {
        None
    };

    // OpenTelemetry Layer
    let telemetry_layer = if enable_opentelemetry {
        let tracer = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .build()
            .tracer("nym-vpnd");
        Some(tracing_opentelemetry::layer().with_tracer(tracer))
    } else {
        None
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(android_layer)
        .with(os_logger)
        .with(json_layer)
        .with(file_layer_json)
        .with(file_layer_plain)
        .with(console_layer_json)
        .with(console_layer_plain)
        .with(sentry_layer)
        .with(telemetry_layer)
        .init();

    if let Some(err) = file_appender_error {
        trace_err_chain!(err, "failed to set up file logging");
    }

    log_panics::init();
    logging_setup
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
