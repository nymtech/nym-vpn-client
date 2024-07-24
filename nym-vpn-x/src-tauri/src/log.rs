use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};
use tauri::api::path::cache_dir;
use tracing_appender::{non_blocking::WorkerGuard, rolling};

use crate::{envi, fs::util::check_dir, APP_DIR};

const ENV_LOG_FILE: &str = "LOG_FILE";
const LOG_DIR: &str = "log";
const LOG_FILE: &str = "app.log";

fn rotate_log_file(cache_dir: PathBuf) -> Result<()> {
    let log_file = cache_dir.join(LOG_FILE);
    if log_file.is_file() {
        let old_file = cache_dir.join(format!("{}.old", LOG_FILE));
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
    }
    Ok(())
}

pub async fn setup_tracing() -> Result<Option<WorkerGuard>> {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env()
        .unwrap()
        .add_directive("hyper::proto=info".parse().unwrap())
        .add_directive("netlink_proto=info".parse().unwrap());

    if envi::is_truthy(ENV_LOG_FILE) {
        let cache_dir = cache_dir().ok_or(anyhow!("Failed to retrieve cache directory path"))?;
        let cache_dir = cache_dir.join(format!("{}/{}", APP_DIR, LOG_DIR));
        check_dir(&cache_dir).await?;
        rotate_log_file(cache_dir.clone()).ok();

        let appender = rolling::never(cache_dir, LOG_FILE);
        let (writer, _guard) = tracing_appender::non_blocking(appender);

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .compact()
            .with_writer(writer)
            .init();
        Ok(Some(_guard))
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .compact()
            .init();
        Ok(None)
    }
}
