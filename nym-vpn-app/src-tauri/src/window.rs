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
// ⚠ keep those in sync with the theme definition in `src/styles.css`
const BG_COLOR_LIGHT: [u8; 3] = [235, 238, 244]; // #ebeef4
const BG_COLOR_DARK: [u8; 3] = [36, 43, 45]; // #242b2d

pub struct AppWindow(pub WebviewWindow);

#[derive(Serialize, Deserialize, Debug, Default, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
enum UiTheme {
    #[default]
    System,
    Light,
    Dark,
}

/// concrete UI mode
#[derive(Serialize, Deserialize, Debug, Default, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
enum UiMode {
    #[default]
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
        // we don't show the window on creation
        .visible(false)
        .title(APP_NAME)
        .fullscreen(false)
        .resizable(true)
        .maximizable(false)
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
    pub fn set_bg_color(&self, db: &Db) -> Result<()> {
        let ui_mode = self.get_current_theme(db).unwrap_or_default();
        let color = match ui_mode {
            UiMode::Light => Color::from(BG_COLOR_LIGHT),
            UiMode::Dark => Color::from(BG_COLOR_DARK),
        };
        debug!("set webview background color to {:?}", color);
        self.0
            .set_background_color(Some(color))
            .inspect_err(|e| error!("failed to set background color: {e}"))?;
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
            .map(AppWindow)
            .or_else(|| {
                debug!("main window not found, re-creating it");
                AppWindow::create_main_window(app).ok()
            })
            .ok_or_else(|| anyhow!("failed to get window {}", label))?;
        Ok(window)
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

    /// retrieve the current theme from the saved settings if any
    /// or fallback to the system theme
    /// defaults to `Light`
    #[instrument(skip_all)]
    fn get_current_theme(&self, db: &Db) -> Result<UiMode> {
        let ui_theme = db
            .get_typed::<UiTheme>(Key::UiTheme.as_ref())?
            .unwrap_or_default();
        Ok(match ui_theme {
            UiTheme::Light => UiMode::Light,
            UiTheme::Dark => UiMode::Dark,
            UiTheme::System => self
                .0
                .theme()
                .inspect_err(|e| {
                    error!("failed to get current window theme: {e}, fallback to `Light`");
                })
                .unwrap_or(Theme::Light)
                .into(),
        })
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

impl From<Theme> for UiMode {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Dark => UiMode::Dark,
            _ => UiMode::Light,
        }
    }
}
