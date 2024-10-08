use crate::db::DbError;
use crate::MAIN_WINDOW_LABEL;

use anyhow::Result;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tracing::{error, info, instrument, warn};
use ts_rs::TS;

pub static STARTUP_ERROR: OnceCell<StartupError> = OnceCell::new();
const WIN_LABEL: &str = "error";
const WIN_TITLE: &str = "NymVPN - Startup error";

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export, export_to = "StartupErrorKey.ts")]
pub enum ErrorKey {
    /// At startup, failed to open the embedded db, generic
    StartupOpenDb,
    /// At startup, failed to open the embedded db because it is already locked
    StartupOpenDbLocked,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct StartupError {
    pub key: ErrorKey,
    pub details: Option<String>,
}

impl StartupError {
    pub fn new(key: ErrorKey, details: Option<String>) -> Self {
        Self { key, details }
    }
}

pub fn set_error(key: ErrorKey, details: Option<&str>) {
    STARTUP_ERROR
        .set(StartupError::new(key, details.map(String::from)))
        .inspect_err(|_| {
            warn!("failed to set startup error: already set");
        })
        .ok();
}

struct WinSizes {
    // (width, height)
    inner: (f64, f64),
    min: (f64, f64),
    max: (f64, f64),
}

#[instrument(skip(app))]
pub fn show_window(app: &AppHandle) -> Result<()> {
    info!("hide the main window");
    let main_win = app.get_webview_window(MAIN_WINDOW_LABEL).unwrap();
    main_win
        .hide()
        .inspect_err(|e| warn!("failed to hide main window: {}", e))
        .ok();

    #[cfg(windows)]
    let sizes = WinSizes {
        inner: (360.0, 380.0),
        min: (260.0, 280.0),
        max: (700.0, 720.0),
    };
    #[cfg(not(windows))]
    let sizes = WinSizes {
        inner: (480.0, 510.0),
        min: (260.0, 280.0),
        max: (900.0, 920.0),
    };
    let window = tauri::WebviewWindowBuilder::new(
        app,
        WIN_LABEL,
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title(WIN_TITLE)
    .fullscreen(false)
    .resizable(true)
    .maximizable(false)
    .visible(false)
    .center()
    .focused(true)
    .inner_size(sizes.inner.0, sizes.inner.1)
    .min_inner_size(sizes.min.0, sizes.min.1)
    .max_inner_size(sizes.max.0, sizes.max.1)
    .build()
    .inspect_err(|e| {
        error!("failed to build the error window: {e}");
    })?;

    let handle = app.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { .. } = event {
            handle.exit(0);
        }
    });

    Ok(())
}

impl From<&DbError> for ErrorKey {
    fn from(value: &DbError) -> Self {
        match value {
            DbError::Locked(_) => ErrorKey::StartupOpenDbLocked,
            _ => ErrorKey::StartupOpenDb,
        }
    }
}
