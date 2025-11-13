use crate::db::DbError;
use crate::window::WindowInitEnv;
use crate::{ERROR_WINDOW_LABEL, MAIN_WINDOW_LABEL};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tracing::{error, info, instrument, warn};
use ts_rs::TS;

const WIN_TITLE: &str = "NymVPN - Error";

#[derive(Debug, Serialize, Deserialize, TS, Clone, strum::AsRefStr)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ErrorKey {
    Internal,
    /// Failed to open the embedded db, generic
    DbOpen,
    /// Failed to open the embedded db because it is already locked
    DbLocked,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export, export_to = "tauri.ts")]
pub struct StartupError {
    #[ts(inline)]
    pub key: ErrorKey,
    pub detail: Option<String>,
}

impl StartupError {
    pub fn new(key: ErrorKey, detail: Option<String>) -> Self {
        Self { key, detail }
    }
}

struct WinSizes {
    // (width, height)
    inner: (f64, f64),
    min: (f64, f64),
    max: (f64, f64),
}

// NOTE: the error window is created here but frontend is
// responsible for showing it
#[instrument(skip(app))]
pub fn create_window(app: &AppHandle, error: StartupError) -> Result<()> {
    info!("hide the main window");
    if let Some(win) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        win.hide()
            .inspect_err(|e| warn!("failed to hide main window: {}", e))
            .ok();
    }

    #[cfg(windows)]
    let sizes = WinSizes {
        inner: (360.0, 400.0),
        min: (260.0, 280.0),
        max: (700.0, 720.0),
    };
    #[cfg(not(windows))]
    let sizes = WinSizes {
        inner: (480.0, 600.0),
        min: (260.0, 280.0),
        max: (1000.0, 1000.0),
    };
    let env = WindowInitEnv::new(false, Some(error)).to_json();
    let window = tauri::WebviewWindowBuilder::new(
        app,
        ERROR_WINDOW_LABEL,
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
    .initialization_script(format!("window._APP = {env};"))
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
            DbError::Locked(_) => ErrorKey::DbLocked,
            DbError::Db(_) => ErrorKey::DbOpen,
            _ => ErrorKey::Internal,
        }
    }
}
