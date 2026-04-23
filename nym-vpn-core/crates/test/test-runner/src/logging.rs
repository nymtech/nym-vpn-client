// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::LazyLock,
};
use test_rpc::logging::{Error, LogFile, LogOutput, Output};
use tokio::{
    fs::File,
    io::{self, AsyncBufReadExt, BufReader},
    sync::{
        Mutex,
        broadcast::{Receiver, Sender, channel},
    },
};

const MAX_OUTPUT_BUFFER: usize = 10_000;
/// Only consider files that end with ".log"
const INCLUDE_LOG_FILE_EXT: &str = "log";
/// Ignore log files that contain ".old"
const EXCLUDE_LOG_FILE_CONTAIN: &str = ".old";
/// Maximum number of lines that each log file may contain
const TRUNCATE_LOG_FILE_LINES: usize = 200;

pub static LOGGER: LazyLock<StdOutBuffer> = LazyLock::new(|| {
    let (sender, listener) = channel(MAX_OUTPUT_BUFFER);
    StdOutBuffer(Mutex::new(listener), sender)
});

pub struct StdOutBuffer(pub Mutex<Receiver<Output>>, pub Sender<Output>);

impl log::Log for StdOutBuffer {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            match record.metadata().level() {
                Level::Error => {
                    self.1
                        .send(Output::Error(format!("{}", record.args())))
                        .unwrap();
                }
                Level::Warn => {
                    self.1
                        .send(Output::Warning(format!("{}", record.args())))
                        .unwrap();
                }
                Level::Info if !record.metadata().target().contains("tarpc") => {
                    self.1
                        .send(Output::Info(format!("{}", record.args())))
                        .unwrap();
                }
                Level::Info => (),
                _ => (),
            }
            println!("{}", record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init_logger() -> Result<(), SetLoggerError> {
    log::set_logger(&*LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}

pub async fn get_nym_app_logs() -> LogOutput {
    LogOutput {
        settings_json: read_settings_file_nym().await,
        log_files: read_log_files_nym().await,
    }
}

async fn read_settings_file_nym() -> Result<String, Error> {
    let settings_path =
        get_default_settings_path_nym().map_err(|error| Error::Logs(format!("{error}")))?;
    read_truncated(&settings_path, None).await.map_err(|error| {
        Error::Logs(format!(
            "Failed to read (truncated) logs from {}: {}",
            settings_path.display(),
            error
        ))
    })
}

async fn read_log_files_nym() -> Result<Vec<Result<LogFile, Error>>, Error> {
    let log_dir = get_default_log_dir_nym().map_err(|error| Error::Logs(format!("{error}")))?;
    let log_dir_path = log_dir.display().to_string();
    let paths = list_logs(log_dir).await.map_err(|error| {
        Error::Logs(format!(
            "Failed to list logs from path {}: {}",
            log_dir_path, error
        ))
    })?;
    let mut log_files = Vec::new();
    for path in paths {
        let log_file = read_truncated(&path, Some(TRUNCATE_LOG_FILE_LINES))
            .await
            .map_err(|error| Error::Logs(format!("{}: {}", path.display(), error)))
            .map(|content| LogFile {
                content,
                name: path,
            });
        log_files.push(log_file);
    }
    Ok(log_files)
}

// use nym_vpnd::service::DEFAULT_LOG_DIR;
#[cfg(unix)]
fn get_default_settings_path_nym() -> Result<PathBuf, Error> {
    // defined in nym-vpnd-lib
    Ok(PathBuf::from("/etc/nym").join("config.json"))
}

#[cfg(unix)]
fn get_default_log_dir_nym() -> Result<PathBuf, Error> {
    Ok(PathBuf::from("/var/log/nym-vpnd"))
}

#[cfg(target_os = "windows")]
fn get_default_settings_path_nym() -> Result<PathBuf, Error> {
    // TODO: Determine correct Windows settings path for nym-vpnd
    Ok(PathBuf::from(r"C:\ProgramData\nym-vpnd\config\config.json"))
}

#[cfg(target_os = "windows")]
fn get_default_log_dir_nym() -> Result<PathBuf, Error> {
    // TODO: Determine correct Windows log directory for nym-vpnd
    Ok(PathBuf::from(r"C:\ProgramData\nym-vpnd\log"))
}

async fn list_logs<T: AsRef<Path>>(log_dir: T) -> Result<Vec<PathBuf>, Error> {
    let mut dir_entries = tokio::fs::read_dir(&log_dir)
        .await
        .map_err(|e| Error::Logs(format!("{}: {}", log_dir.as_ref().display(), e)))?;

    let mut paths = Vec::new();
    while let Ok(Some(entry)) = dir_entries.next_entry().await {
        let path = entry.path();
        if let Some(u8_path) = path.to_str()
            && u8_path.contains(EXCLUDE_LOG_FILE_CONTAIN)
        {
            continue;
        }

        if path.extension() == Some(OsStr::new(INCLUDE_LOG_FILE_EXT)) {
            paths.push(path);
        }
    }
    Ok(paths)
}

/// Read the contents of a file to string, optionally truncating the result by given amount of
/// lines.
async fn read_truncated<T: AsRef<Path>>(
    path: T,
    truncate_lines: Option<usize>,
) -> io::Result<String> {
    let mut output = vec![];
    let reader = BufReader::new(File::open(path).await?);
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        output.push(line);
    }
    if let Some(max_number_of_lines) = truncate_lines
        && output.len() > max_number_of_lines
    {
        output = output.split_off(output.len() - max_number_of_lines);
    }

    Ok(output.join("\n"))
}
