use std::{fs, path::PathBuf};

use crate::envi;
use crate::fs::path::LOG_DIR_PATH;
use anyhow::{anyhow, Result};
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::fmt::writer::MakeWriterExt;

const ENV_LOG_FILE: &str = "LOG_FILE";
const LOG_FILE: &str = "app.log";
const LOG_FILE_OLD: &str = "app.old.log";

fn rotate_log_file(log_dir: PathBuf) -> Result<()> {
    let log_file = log_dir.join(LOG_FILE);
    if log_file.is_file() {
        let old_file = log_dir.join(LOG_FILE_OLD);
        let data = fs::read(&log_file).inspect_err(|e| {
            eprintln!(
                "failed to read log file during log rotation {}: {e}",
                log_file.display()
            )
        })?;
        fs::write(&old_file, data).inspect_err(|e| {
            eprintln!(
                "failed to write log file during log rotation {}: {e}",
                old_file.display()
            )
        })?;
        fs::remove_file(log_file)?;
    }
    Ok(())
}

pub async fn setup_tracing(log_file: bool) -> Result<Option<WorkerGuard>> {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env()
        .unwrap()
        .add_directive("hyper::proto=info".parse().unwrap())
        .add_directive("netlink_proto=info".parse().unwrap());

    if log_file || envi::is_truthy(ENV_LOG_FILE) {
        let log_dir = LOG_DIR_PATH
            .clone()
            .ok_or(anyhow!("Failed to retrieve log directory path"))?;
        rotate_log_file(log_dir.clone()).ok();

        let appender = rolling::never(log_dir, LOG_FILE);
        let (writer, guard) = tracing_appender::non_blocking(appender);

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .compact()
            .with_ansi(false)
            .with_writer(std::io::stdout.and(writer))
            .init();
        Ok(Some(guard))
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .compact()
            .init();
        Ok(None)
    }
}
