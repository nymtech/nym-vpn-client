use crate::fs::path::APP_LOG_DIR;
use crate::{env, Cli};
use std::io::{self, IsTerminal};
use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};
use tracing::{debug, info};
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

const ENV_LOG_FILE: &str = "LOG_FILE";
const LOG_FILE: &str = "app.log";
const LOG_FILE_OLD: &str = "app.old.log";

fn rotate_log_file(log_dir: PathBuf) -> Result<Option<PathBuf>> {
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

pub async fn setup_tracing(cli: &Cli) -> Result<Option<WorkerGuard>> {
    let mut filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()?
        .add_directive("hyper::proto=info".parse()?)
        .add_directive("netlink_proto=info".parse()?);

    if let Some(log_level) = cli.log_level.as_ref() {
        filter =
            filter.add_directive(format!("{}={}", env!("CARGO_CRATE_NAME"), log_level).parse()?);
    }

    let mut layers = Vec::new();
    let mut worker = None;
    let mut log_file = None;
    let mut old_file = None;
    #[cfg(windows)]
    let enable_ansi = !cli.console;
    #[cfg(not(windows))]
    let enable_ansi = true;

    let stdout_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_ansi(enable_ansi);

    if cli.log_file || env::is_truthy(ENV_LOG_FILE) {
        let log_dir = APP_LOG_DIR
            .clone()
            .ok_or(anyhow!("failed to get log dir"))?;
        log_file = Some(log_dir.join(LOG_FILE));
        old_file = rotate_log_file(log_dir.clone()).ok().flatten();

        let appender = rolling::never(log_dir.clone(), LOG_FILE);
        let (writer, guard) = tracing_appender::non_blocking(appender);
        worker = Some(guard);

        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(writer)
            .compact()
            .with_ansi(false);
        layers.push(file_layer.boxed());

        if io::stdout().is_terminal() {
            layers.push(stdout_layer.boxed());
        }
    } else {
        layers.push(stdout_layer.boxed());
    }

    tracing_subscriber::registry()
        .with(filter)
        .with(layers)
        .init();

    if let Some(file) = old_file {
        debug!("rotated log file: {}", file.display());
    }
    if let Some(file) = log_file {
        info!("logging to file: {}", file.display());
    }

    Ok(worker)
}
