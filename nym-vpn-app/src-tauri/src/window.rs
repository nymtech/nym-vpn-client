use crate::cli::Cli;
use crate::db::{Db, Key};
use crate::env::{DEV_MODE, UPDATER_ENABLED};
use crate::startup_error::StartupError;
use crate::state::app::VpnMode;
#[cfg(target_os = "linux")]
use crate::sys::DisplayServer;
use crate::sys::OsInfo;
use crate::{
    APP_NAME, DEFAULT_DOMAIN_FRONTING, DEFAULT_NETSTATS_ENABLED, DEFAULT_QUIC,
    DEFAULT_SENTRY_ENABLED, ENV_APP_NOSPLASH, MAIN_WINDOW_LABEL, env,
};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use tauri::window::Color;
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Theme,
    WebviewUrl, WebviewWindow, WebviewWindowBuilder, Window, WindowEvent,
};
use tracing::{debug, error, instrument, trace, warn};
use ts_rs::TS;

const MAIN_WEBVIEW_URL: &str = "index.html";
// ⚠ keep those in sync with the theme definition in `src/styles.css`
const BG_COLOR_LIGHT: [u8; 3] = [235, 238, 244]; // #ebeef4
const BG_COLOR_DARK: [u8; 3] = [36, 43, 45]; // #242b2d

pub struct AppWindow(pub WebviewWindow);

#[derive(Serialize, Deserialize, Debug, Default, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "tauri.ts")]
enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

/// concrete UI mode
#[derive(Serialize, Deserialize, Debug, Default, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "tauri.ts")]
enum UiTheme {
    #[default]
    Light,
    Dark,
}

#[derive(Serialize, Deserialize, Debug, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts", rename = "JsEnv")]
pub struct WindowInitEnv {
    pub dev_mode: bool,
    pub updater_enabled: bool,
    pub no_splash: bool,
    #[ts(inline)]
    pub default_vpn_mode: VpnMode,
    pub default_sentry: bool,
    pub default_netstats: bool,
    pub default_quic: bool,
    pub default_domain_fronting: bool,
    pub startup_error: Option<StartupError>,
}

impl AppWindow {
    #[instrument(skip(app))]
    pub fn create_main_window(app: &AppHandle, cli: &Cli) -> Result<AppWindow> {
        let no_splash = cli.nosplash || env::is_truthy(ENV_APP_NOSPLASH);
        let win_env = WindowInitEnv::new(no_splash, None).to_json();
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
        .min_inner_size(328.0, 200.0)
        .max_inner_size(800.0, 1400.0)
        .initialization_script(format!("window._APP = {win_env};"))
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
            UiTheme::Light => Color::from(BG_COLOR_LIGHT),
            UiTheme::Dark => Color::from(BG_COLOR_DARK),
        };
        debug!("set webview background color to {:?}", color);
        self.0
            .set_background_color(Some(color))
            .inspect_err(|e| error!("failed to set background color: {e}"))?;
        Ok(())
    }

    /// try to get the window, if not found create it from its config
    #[instrument(skip(app))]
    pub fn get_or_create(app: &AppHandle, label: &str) -> Result<Self> {
        let cli = app
            .try_state::<Cli>()
            .map(|s| s.inner().clone())
            .unwrap_or_default();
        let window = app
            .get_webview_window(label)
            .map(AppWindow)
            .or_else(|| {
                debug!("main window not found, creating it");
                AppWindow::create_main_window(app, &cli).ok()
            })
            .ok_or_else(|| {
                error!("failed to get window {label}");
                anyhow!("failed to get window {label}")
            })?;
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

    #[instrument(skip_all)]
    pub fn set_max_size(
        &self,
        #[cfg(target_os = "linux")] display_server: Option<DisplayServer>,
    ) -> Result<()> {
        let Some(monitor) = self.0.current_monitor().inspect_err(|e| {
            error!("failed to get current monitor: {e}");
        })?
        else {
            #[cfg(target_os = "linux")]
            {
                // On Wayland it is expected failing to detected monitor info
                // especially when the window is not yet visible
                if display_server == Some(DisplayServer::Wayland) {
                    tracing::info!("failed to get current monitor details");
                } else {
                    warn!("failed to get current monitor details");
                }
            }
            #[cfg(not(target_os = "linux"))]
            warn!("failed to get current monitor details");
            return Ok(());
        };
        // in case of monitor > 1440p, increase the max allowed window size
        if monitor.size().width > 2560 {
            debug!("setting max window size to 1000x1600");
            self.0
                .set_max_size(Some(PhysicalSize::new(1000, 1600)))
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
    fn get_current_theme(&self, db: &Db) -> Result<UiTheme> {
        let ui_theme = db
            .get_typed::<ThemeMode>(Key::UiTheme.as_ref())?
            .unwrap_or(ThemeMode::System);
        Ok(match ui_theme {
            ThemeMode::Light => UiTheme::Light,
            ThemeMode::Dark => UiTheme::Dark,
            ThemeMode::System => self
                .0
                .theme()
                .inspect(|theme| {
                    trace!("current window theme: {theme}");
                })
                .inspect_err(|e| {
                    error!("failed to get current window theme: {e}, fallback to `Light`");
                })
                .unwrap_or(Theme::Light)
                .into(),
        })
    }
}

#[instrument(skip(os, win))]
pub fn handle_event(#[allow(unused_variables)] os: &OsInfo, win: &Window, event: &WindowEvent) {
    // keep the app running in the background on window close request
    if let WindowEvent::CloseRequested { api, .. } = event
        && win.label() == MAIN_WINDOW_LABEL
    {
        win.hide()
            .inspect_err(|e| error!("failed to hide main window: {e}"))
            .ok();
        api.prevent_close();
    }
    if let WindowEvent::Focused(true) = event
        && win.label() == MAIN_WINDOW_LABEL
    {
        #[cfg(target_os = "linux")]
        {
            // credits @stenya
            // https://github.com/safing/portmaster/commit/95838b510c75fa9dde6e99a4492e1c7e34f7cf18

            // Workaround for KDE/Wayland environments on Linux:
            // On KDE with Wayland, after hiding and showing the window,
            // the title-bar buttons (close, minimize, maximize) may stop working.
            // Toggling the resizable property appears to resolve this issue.
            // see https://github.com/safing/portmaster/issues/1909
            // https://github.com/tauri-apps/tauri/issues/6162#issuecomment-1423304398
            if os.display_server == Some(DisplayServer::Wayland) {
                trace!("toggle resizable");
                win.set_resizable(false).ok();
                win.set_resizable(true).ok();
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(tag = "type")]
pub enum WindowSize {
    Physical { width: u32, height: u32 },
    Logical { width: f64, height: f64 },
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export, export_to = "tauri.ts")]
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
    if let Ok(win) = AppWindow::get_or_create(app, MAIN_WINDOW_LABEL) {
        win.wake_up();
    }
}

impl From<Theme> for UiTheme {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Dark => UiTheme::Dark,
            _ => UiTheme::Light,
        }
    }
}

impl WindowInitEnv {
    pub fn new(no_splash: bool, startup_error: Option<StartupError>) -> Self {
        WindowInitEnv {
            dev_mode: *DEV_MODE,
            updater_enabled: *UPDATER_ENABLED,
            no_splash,
            default_vpn_mode: Default::default(),
            default_sentry: DEFAULT_SENTRY_ENABLED,
            default_netstats: DEFAULT_NETSTATS_ENABLED,
            default_quic: DEFAULT_QUIC,
            default_domain_fronting: DEFAULT_DOMAIN_FRONTING,
            startup_error,
        }
    }

    #[instrument]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self)
            .inspect_err(|e| error!("failed to serialize as JSON string: {e}"))
            .unwrap_or_else(|_| "{}".to_string())
    }
}
