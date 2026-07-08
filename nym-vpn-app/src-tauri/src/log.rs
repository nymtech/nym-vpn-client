use crate::Cli;
use crate::fs::path::APP_LOG_DIR;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use sentry::integrations::tracing as sentry_tracing;
use tracing::{Level, debug, error, info};
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::reload;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

const LOG_FILE: &str = "app.log";
const LOG_FILE_OLD: &str = "app.old.log";

/// Closure that toggles the file logging layer at runtime.
/// Returns the new [`WorkerGuard`] when enabling (which must be kept alive
/// while logging to file), or `None` when disabling.
type ApplyFn = Box<dyn Fn(bool) -> Result<Option<WorkerGuard>> + Send + Sync>;

/// Runtime control for app file logging.
///
/// The tracing subscriber is initialized once at startup with a reloadable
/// layer. Toggling this control hot-swaps the file layer in place.
pub struct DebugLogging {
    apply: ApplyFn,
    /// Keeps the non-blocking appender's worker thread alive while enabled.
    /// Dropping it flushes and stops the writer.
    guard: Option<WorkerGuard>,
    enabled: bool,
}

impl DebugLogging {
    fn new(apply: ApplyFn) -> Self {
        DebugLogging {
            apply,
            guard: None,
            enabled: false,
        }
    }

    pub fn set(&mut self, enabled: bool) -> Result<()> {
        if self.enabled == enabled {
            return Ok(());
        }
        self.guard = (self.apply)(enabled)?;
        self.enabled = enabled;
        Ok(())
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

impl std::fmt::Debug for DebugLogging {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugLogging")
            .field("enabled", &self.enabled)
            .finish()
    }
}

fn rotate_log_file(log_dir: &Path) -> Result<Option<PathBuf>> {
    let log_file = log_dir.join(LOG_FILE);
    if log_file.is_file() {
        let old_file = log_dir.join(LOG_FILE_OLD);
        fs::rename(&log_file, &old_file).inspect_err(|e| {
            eprintln!(
                "failed to rename log file during log rotation {}: {e}",
                log_file.display()
            )
        })?;
        return Ok(Some(old_file));
    }
    Ok(None)
}

pub async fn setup_tracing(
    #[cfg_attr(not(windows), allow(unused_variables))] cli: &Cli,
    sentry_enabled: bool,
    debug_logging: bool,
) -> Result<DebugLogging> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env()?
        .add_directive("hyper::proto=info".parse()?)
        .add_directive("netlink_proto=info".parse()?);

    #[cfg(windows)]
    let enable_ansi = !cli.console;
    #[cfg(not(windows))]
    let enable_ansi = true;

    let stdout_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_ansi(enable_ansi);

    let none_layer: Option<Box<dyn Layer<_> + Send + Sync>> = None;
    let (file_reload_layer, reload_handle) = reload::Layer::new(none_layer);

    let sentry_layer = sentry_enabled.then(|| {
        sentry_tracing::layer().event_filter(|md| match md.level() {
            &Level::ERROR | &Level::WARN => {
                sentry_tracing::EventFilter::Event | sentry_tracing::EventFilter::Log
            }
            &Level::INFO => {
                sentry_tracing::EventFilter::Breadcrumb | sentry_tracing::EventFilter::Log
            }
            _ => sentry_tracing::EventFilter::Ignore,
        })
    });

    tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .with(file_reload_layer)
        .with(sentry_layer)
        .init();

    let apply: ApplyFn = Box::new(move |enabled: bool| -> Result<Option<WorkerGuard>> {
        if enabled {
            let log_dir = APP_LOG_DIR
                .clone()
                .ok_or(anyhow!("failed to get log dir"))?;
            if let Some(old) = rotate_log_file(&log_dir).ok().flatten() {
                debug!("rotated log file: {}", old.display());
            }
            let appender = rolling::never(&log_dir, LOG_FILE);
            let (writer, guard) = tracing_appender::non_blocking(appender);
            let layer = tracing_subscriber::fmt::layer()
                .with_writer(writer)
                .compact()
                .with_ansi(false)
                .boxed();
            reload_handle
                .reload(Some(layer))
                .map_err(|e| anyhow!("failed to enable file logging: {e}"))?;
            info!(
                "app file logging enabled: {}",
                log_dir.join(LOG_FILE).display()
            );
            Ok(Some(guard))
        } else {
            reload_handle
                .reload(None)
                .map_err(|e| anyhow!("failed to disable file logging: {e}"))?;
            info!("app file logging disabled");
            Ok(None)
        }
    });

    let mut control = DebugLogging::new(apply);
    if debug_logging {
        // File logging is optional; if it can't be initialized, log and fall
        // back to disabled rather than aborting app startup.
        if let Err(e) = control.set(true) {
            error!("failed to enable app file logging at startup, continuing without it: {e}");
        }
    }

    Ok(control)
}
