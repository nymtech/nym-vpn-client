// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, sync::OnceLock};

use nym_vpn_lib::logging::LoggingSetup;
use sentry::ClientInitGuard;

static SHARED_STATE: OnceLock<State> = OnceLock::new();

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

#[allow(non_snake_case)]
#[uniffi::export]
pub fn initLogger(log_dir: Option<PathBuf>, log_level: LogLevel, sentry_monitoring: bool) {
    let _ = SHARED_STATE.get_or_init(|| {
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
        });

        tracing::info!(
            "Setting log level: {verbosity_level}, path?: {:?}",
            log_dir.as_ref().map(|path| path.display().to_string())
        );

        nym_vpn_lib::log_software_and_os_version();

        State {
            sentry_init_guard,
            _logging_setup: logging_setup,
        }
    });
}

pub fn is_sentry_enabled() -> bool {
    SHARED_STATE
        .get()
        .is_some_and(|state| state.sentry_init_guard.is_some())
}
