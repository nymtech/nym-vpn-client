use crate::error::BackendError;
use serde::Serialize;
use ts_rs::TS;

#[cfg(any(target_os = "windows", target_os = "linux"))]
mod utils;

#[cfg(windows)]
mod windows_discovery;

#[cfg(target_os = "linux")]
mod linux_discovery;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "tauri.ts")]
pub struct App {
    pub name: String,
    /// Absolute path to the main executable.
    pub executable_path: String,
    /// Absolute path to the cached icon PNG, when available. Stored in tauri app cache directory.
    pub icon: Option<String>,
}

/// Return all installed applications on the current platform.
pub fn get_installed_apps(_app: tauri::AppHandle) -> Result<Vec<App>, BackendError> {
    #[cfg(target_os = "windows")]
    {
        windows_discovery::get_windows_apps(_app)
    }

    #[cfg(target_os = "linux")]
    {
        linux_discovery::get_linux_apps()
    }

    #[cfg(target_os = "macos")]
    {
        Ok(vec![])
    }
}
