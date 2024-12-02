use crate::db::{Db, Key};
use crate::{APP_NAME, MAIN_WINDOW_LABEL};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tauri::window::Color;
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Theme,
    WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};
use tracing::{debug, error, instrument, warn};
use ts_rs::TS;

const MAIN_WEBVIEW_URL: &str = "index.html";

pub struct AppWindow(pub WebviewWindow);

#[derive(Deserialize, Debug)]
enum UiTheme {
    System,
    Light,
    Dark,
}

enum UiMode {
    Light,
    Dark,
}

impl AppWindow {
    #[instrument(skip(app))]
    pub fn create_main_window(app: &AppHandle) -> Result<AppWindow> {
        let window = WebviewWindowBuilder::new(
            app,
            MAIN_WINDOW_LABEL,
            WebviewUrl::App(MAIN_WEBVIEW_URL.into()),
        )
        .title(APP_NAME)
        .background_color(Color::from((255, 255, 255)))
        .fullscreen(false)
        .resizable(true)
        .maximizable(false)
        .visible(false)
        .center()
        .focused(true)
        .inner_size(328.0, 710.0)
        .min_inner_size(160.0, 346.0)
        .max_inner_size(600.0, 1299.0)
        .build()
        .inspect_err(|e| error!("failed to create main window: {e}"))?;
        Ok(AppWindow(window))
    }

    /// set the background color of the webview window from saved
    /// theme settings (if any)
    #[instrument(skip_all)]
    pub fn set_bg_color(&mut self, db: &Db) -> Result<()> {
        let ui_theme: Option<UiTheme> = db.get_typed::<UiTheme>(Key::UiTheme)?;


        let mut color = "#ffffff";
        match ui_theme {
            Some(UiTheme::Dark) => color = "#000000",
            Some(UiTheme::Light) => color = "#ffffff",
            _ => {
                let current_theme = self.0.theme().inspect_err(|e| {
                    error!("failed to get current window theme: {e}");
                })?;
            }
        }
        // if let Some(theme) = theme {
            // let theme: Theme = serde_json::from_value(theme)?;
            // let color = theme.background_color;
            // self.0
            //     .set_background_color(Color::from((color.r, color.g, color.b)))
            //     .inspect_err(|e| error!("failed to set background color: {e}"))
            //     .map_err(|e| anyhow!("failed to set background color: {e}"))?;
        // } else {
            // let current_theme = self.0.theme().inspect_err(|e| {
            //     error!("failed to get current window theme: {e}");
            // })?;
            // match current_theme {
            //     tauri::window::Theme::Light => {
            //         self.0
            //             .set_background_color(Color::from((255, 255, 255)))
            //             .inspect_err(|e| error!("failed to set background color: {e}"))
            //             .map_err(|e| anyhow!("failed to set background color: {e}"))?;
            //     }
            //     tauri::window::Theme::Dark => {
            //         self.0
            //             .set_background_color(Color::from((0, 0, 0)))
            //             .inspect_err(|e| error!("failed to set background color: {e}"))
            //             .map_err(|e| anyhow!("failed to set background color: {e}"))?;
            //     }
            //     _ => {}
            // }
        // }
        Ok(())
    }

    #[instrument(skip(app))]
    pub fn get(app: &AppHandle, label: &str) -> Result<Self> {
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
                // TODO this will not work anymore as there is no longer a WindowConfig declared
                //  in the tauri.conf.json; call `create_main_window` instead
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
    if let Ok(win) = AppWindow::get(app, MAIN_WINDOW_LABEL) {
        win.wake_up();
    } else {
        error!("failed to get window {}", MAIN_WINDOW_LABEL);
    }
}
