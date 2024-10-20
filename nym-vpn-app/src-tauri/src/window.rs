use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition, PhysicalSize,
    WebviewWindow, WebviewWindowBuilder,
};
use tracing::{debug, error, instrument, warn};
use ts_rs::TS;

use crate::MAIN_WINDOW_LABEL;

pub struct AppWindow(pub WebviewWindow);

impl AppWindow {
    #[instrument(skip(app))]
    pub fn new(app: &AppHandle, label: &str) -> Result<Self> {
        Ok(AppWindow(app.get_webview_window(label).ok_or_else(
            || {
                error!("failed to get window {}", label);
                anyhow!("failed to get window {}", label)
            },
        )?))
    }

    /// try to get the window, if not found recreate it from its config
    #[instrument(skip(app))]
    pub fn get_or_create(app: &AppHandle, label: &str) -> Result<Self> {
        let window = app
            .get_webview_window(label)
            .or_else(|| {
                debug!("main window not found, re-creating it");
                app.config()
                    .app
                    .windows
                    .iter()
                    .find(|cfg| cfg.label == label)
                    .or_else(|| {
                        error!("window config not found for label {}", label);
                        None
                    })
                    .and_then(|cfg| {
                        WebviewWindowBuilder::from_config(app, cfg)
                            .inspect_err(|e| {
                                error!("failed to create window builder from config: {e}")
                            })
                            .ok()
                            .and_then(|b| {
                                b.build()
                                    .inspect_err(|e| error!("failed to create window: {e}"))
                                    .ok()
                            })
                    })
            })
            .ok_or_else(|| anyhow!("failed to get window {}", label))?;
        Ok(AppWindow(window))
    }

    /// "Wake up" the window, show it, unminimize it and focus it
    #[instrument(skip_all)]
    pub fn wake_up(&self) {
        if !self.is_visible() {
            self.0
                .show()
                .inspect_err(|e| error!("failed to show window: {e}"))
                .ok();
        }
        if self.is_minimized() {
            self.0
                .unminimize()
                .inspect_err(|e| error!("failed to unminimize window: {e}"))
                .ok();
        }
        self.0
            .set_focus()
            .inspect_err(|e| error!("failed to focus window: {e}"))
            .ok();
    }

    pub fn is_visible(&self) -> bool {
        self.0.is_visible().ok().unwrap_or(false)
    }

    pub fn is_minimized(&self) -> bool {
        self.0.is_minimized().ok().unwrap_or(false)
    }

    /// remove splash screen from HTML and show main window
    #[instrument(skip_all)]
    pub fn no_splash(&self) {
        self.0
            .eval("document.getElementById('splash').remove();")
            .inspect_err(|e| error!("failed to remove splash screen: {e}"))
            .ok();

        self.0
            .show()
            .inspect_err(|e| error!("failed to show the window: {e}"))
            .ok();
    }

    #[instrument(skip_all)]
    pub fn set_max_size(&self) -> Result<()> {
        let Some(monitor) = self.0.current_monitor().inspect_err(|e| {
            error!("failed to get current monitor: {e}");
        })?
        else {
            warn!("failed to get current monitor details");
            return Ok(());
        };
        // in case of monitor > 1440p, increase the max allowed window size
        if monitor.size().width > 2560 {
            debug!("setting max window size to 739x1600");
            self.0
                .set_max_size(Some(PhysicalSize::new(739, 1600)))
                .inspect_err(|e| {
                    error!("failed to set max size: {e}");
                })
                .map_err(|e| anyhow!("failed to set window max size: {e}"))?;
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum WindowSize {
    Physical { width: u32, height: u32 },
    Logical { width: f64, height: f64 },
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum WindowPosition {
    Physical { x: i32, y: i32 },
    Logical { x: f64, y: f64 },
}

impl From<WindowSize> for tauri::Size {
    fn from(size: WindowSize) -> Self {
        match size {
            WindowSize::Physical { width, height } => {
                tauri::Size::Physical(PhysicalSize::new(width, height))
            }
            WindowSize::Logical { width, height } => {
                tauri::Size::Logical(LogicalSize::new(width, height))
            }
        }
    }
}

impl From<&PhysicalSize<u32>> for WindowSize {
    fn from(size: &PhysicalSize<u32>) -> Self {
        WindowSize::Physical {
            width: size.width,
            height: size.height,
        }
    }
}

impl From<WindowPosition> for tauri::Position {
    fn from(position: WindowPosition) -> Self {
        match position {
            WindowPosition::Physical { x, y } => {
                tauri::Position::Physical(PhysicalPosition::new(x, y))
            }
            WindowPosition::Logical { x, y } => {
                tauri::Position::Logical(LogicalPosition::new(x, y))
            }
        }
    }
}

impl From<&PhysicalPosition<i32>> for WindowPosition {
    fn from(size: &PhysicalPosition<i32>) -> Self {
        WindowPosition::Physical {
            x: size.x,
            y: size.y,
        }
    }
}

#[instrument(skip_all)]
pub fn focus_main_window(app: &AppHandle) {
    if let Ok(win) = AppWindow::new(app, MAIN_WINDOW_LABEL) {
        win.wake_up();
    } else {
        error!("failed to get window {}", MAIN_WINDOW_LABEL);
    }
}
