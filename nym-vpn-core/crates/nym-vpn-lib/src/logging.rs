// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use itertools::Itertools;
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

#[derive(Clone, Debug)]
pub struct FileAppender {
    inner: Arc<Mutex<Option<RollingFileAppender>>>,
    log_dir: PathBuf,
    log_file: String,
}

impl FileAppender {
    /// Create new file appender and making a backup of existing log file
    ///
    /// ## Arguments
    ///
    /// * `log_dir`: Directory where the log files are stored.
    /// * `log_file_name`: Current log file (i.e. "nym_vpn.log")
    /// * `old_log_file_name`: Backup log file (i.e. "nym_vpn.log.old")
    pub fn new(log_dir: PathBuf, log_file_name: &str, old_log_file_name: &str) -> Self {
        let log_file_path = log_dir.join(log_file_name);
        let old_log_file_path = log_dir.join(old_log_file_name);

        if let Err(err) = std::fs::rename(&log_file_path, &old_log_file_path)
            && err.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(
                "Log rotation could not be performed, we're going to just append to the same file"
            );
        }

        let inner = Arc::new(Mutex::new(Some(tracing_appender::rolling::never(
            log_dir.clone(),
            log_file_name,
        ))));

        Self {
            inner,
            log_dir,
            log_file: log_file_name.to_owned(),
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
    let (mut file_writer, worker_guard) = if let Some(log_dir) = options.log_dir {
        let file_appender = FileAppender::new(log_dir, DEFAULT_LOG_FILE, DEFAULT_OLD_LOG_FILE);
        let file_manager = FileManager::new(file_appender.clone());
        let (file_writer, worker_guard) = tracing_appender::non_blocking(file_manager);

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
