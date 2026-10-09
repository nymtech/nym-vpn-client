// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, sync::Mutex};

use nym_vpn_lib::logging::LoggingSetup;
use sentry::ClientInitGuard;

static SHARED_STATE: Mutex<Option<State>> = Mutex::new(None);

struct State {
    sentry_init_guard: Option<ClientInitGuard>,
    _logging_setup: Option<LoggingSetup>,
}

#[derive(Debug, uniffi::Enum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for tracing::Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

#[derive(Debug, uniffi::Object)]
#[uniffi::export(Display)]
pub struct InitLoggerError(nym_vpn_lib::logging::SharedFileAppenderError);

impl From<nym_vpn_lib::logging::SharedFileAppenderError> for InitLoggerError {
    fn from(value: nym_vpn_lib::logging::SharedFileAppenderError) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for InitLoggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for InitLoggerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn initLogger(
    log_dir: Option<PathBuf>,
    log_level: LogLevel,
    sentry_monitoring: bool,
) -> Result<(), InitLoggerError> {
    let mut state = SHARED_STATE.lock().expect("failed to lock on shared state");
    if state.is_some() {
        return Ok(());
    }

    let sentry_init_guard = if sentry_monitoring {
        nym_vpn_lib::sentry::init_sentry()
    } else {
        None
    };

    let verbosity_level = tracing::Level::from(log_level);

    let logging_setup = nym_vpn_lib::logging::setup_logging(nym_vpn_lib::logging::Options {
        verbosity_level,
        enable_stdout_log: false,
        enable_json_log: false,
        log_dir: log_dir.clone(),
        sentry: sentry_monitoring,
    })?;

    tracing::info!(
        "Setting log level: {verbosity_level}, path?: {:?}",
        log_dir.as_ref().map(|path| path.display().to_string())
    );

    nym_vpn_lib::log_software_and_os_version();

    *state = Some(State {
        sentry_init_guard,
        _logging_setup: logging_setup,
    });
    Ok(())
}

pub fn is_sentry_enabled() -> bool {
    SHARED_STATE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .is_some_and(|state| state.sentry_init_guard.is_some())
}
