use std::{fs, path::PathBuf};

use crate::envi;
use crate::fs::path::APP_LOG_DIR;
use anyhow::{anyhow, Result};
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::EnvFilter;

const ENV_LOG_FILE: &str = "LOG_FILE";
const LOG_FILE: &str = "app.log";
const LOG_FILE_OLD: &str = "app.old.log";

fn rotate_log_file(log_dir: PathBuf) -> Result<()> {
    let log_file = log_dir.join(LOG_FILE);
    if log_file.is_file() {
        let old_file = log_dir.join(LOG_FILE_OLD);
        println!("rotate log file to: {}", old_file.display());
        fs::rename(&log_file, &old_file).inspect_err(|e| {
            eprintln!(
                "failed to rename log file during log rotation {}: {e}",
                log_file.display()
            )
        })?;
    }
    Ok(())
}

pub async fn setup_tracing(log_file: bool) -> Result<Option<WorkerGuard>> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()?
        .add_directive("hyper::proto=info".parse()?)
        .add_directive("netlink_proto=info".parse()?);

    if log_file || envi::is_truthy(ENV_LOG_FILE) {
        let log_dir = APP_LOG_DIR
            .clone()
            .ok_or(anyhow!("failed to get log dir"))?;
        let log_file = log_dir.join(LOG_FILE);
        println!("logging to file: {}", log_file.display());
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
