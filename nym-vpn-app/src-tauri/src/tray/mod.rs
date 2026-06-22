use anyhow::Result;
use tauri::{AppHandle, Manager};
use tracing::{info, instrument, trace, warn};

use crate::vpnd::tunnel::TunnelState;
use crate::{
    MAIN_WINDOW_LABEL, state::SharedAppState, vpnd::client::VpndClient, window::AppWindow,
};

#[cfg(not(target_os = "linux"))]
mod desktop;
#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(target_os = "linux"))]
use desktop::Backend;
#[cfg(target_os = "linux")]
use linux::Backend;

/// Platform-agnostic contract that every tray backend implements.
///
/// Both the native Tauri backend (Windows/macOS, see [`desktop`]) and the ksni
/// `StatusNotifierItem` backend (Linux, see [`linux`]) implement this, so the compiler
/// rejects any signature drift between them — even though only one is compiled per target.
///
/// Note this trait is intentionally private: it is consumed only through the [`TrayManager`]
/// façade via static dispatch, so the async methods never need `Send` bounds added by callers.
trait TrayBackend: Sized + Send + Sync {
    fn new(app: &AppHandle) -> Result<Self>;
    async fn update_tray_icon(&self, state: TunnelState);
    async fn update_tray_show_hide(&self, show_hide: String);
    async fn update_tray_quit(&self, quit: String);
    async fn update_tray_mode(&self, mode: String);
    async fn update_tray_state(&self, state: String);
    async fn update_tray_entry(&self, entry: String);
    async fn update_tray_exit(&self, exit: String);
    async fn update_tray_entry_visible(&self, visible: bool);
}

/// The system tray, managed as Tauri state and driven from `commands::tray`.
///
/// Thin façade over the platform-selected [`Backend`]. The delegating bodies need no `cfg`
/// because every backend exposes the same API through [`TrayBackend`]; the platform-specific
/// code lives in the [`desktop`] / [`linux`] modules.
pub struct TrayManager(Backend);

impl TrayManager {
    pub fn new(app: &AppHandle) -> Result<Self> {
        Ok(Self(Backend::new(app)?))
    }

    pub async fn update_tray_icon(&self, state: TunnelState) {
        self.0.update_tray_icon(state).await
    }

    pub async fn update_tray_show_hide(&self, show_hide: String) {
        self.0.update_tray_show_hide(show_hide).await
    }

    pub async fn update_tray_quit(&self, quit: String) {
        self.0.update_tray_quit(quit).await
    }

    pub async fn update_tray_mode(&self, mode: String) {
        self.0.update_tray_mode(mode).await
    }

    pub async fn update_tray_state(&self, state: String) {
        self.0.update_tray_state(state).await
    }

    pub async fn update_tray_entry(&self, entry: String) {
        self.0.update_tray_entry(entry).await
    }

    pub async fn update_tray_exit(&self, exit: String) {
        self.0.update_tray_exit(exit).await
    }

    pub async fn update_tray_entry_visible(&self, visible: bool) {
        self.0.update_tray_entry_visible(visible).await
    }
}

// ---------------------------------------------------------------------------
// Shared, platform-agnostic helpers used by both backends.
// ---------------------------------------------------------------------------

#[instrument(skip(app))]
fn show_window(app: &AppHandle, toggle: bool) -> Result<()> {
    let window = AppWindow::get_or_create(app, MAIN_WINDOW_LABEL)?;

    if !window.is_visible() {
        trace!("showing main window");
        let _ = window
            .0
            .show()
            .inspect_err(|e| warn!("failed to show main window: {e}"));
        let _ = window
            .0
            .set_focus()
            .inspect_err(|e| warn!("failed to focus main window: {e}"));
        return Ok(());
    }

    if window.is_visible() && !window.is_minimized() && toggle {
        trace!("hiding main window");
        let _ = window
            .0
            .hide()
            .inspect_err(|e| warn!("failed to hide main window: {e}"));
        return Ok(());
    }

    if window.is_minimized() {
        trace!("unminimizing main window");
        let _ = window
            .0
            .unminimize()
            .inspect_err(|e| warn!("failed to unminimize main window: {e}"));
        let _ = window
            .0
            .set_focus()
            .inspect_err(|e| warn!("failed to focus main window: {e}"));
        return Ok(());
    }

    let _ = window
        .0
        .set_focus()
        .inspect_err(|e| warn!("failed to focus main window: {e}"));

    Ok(())
}

fn quit_app(app: AppHandle) {
    // Use Tauri's runtime explicitly — on Linux this can be called from ksni's async-io
    // executor thread, which has no tokio context.
    tauri::async_runtime::spawn(async move {
        let state = app.state::<SharedAppState>();
        let vpnd = app.state::<VpndClient>();

        let app_state = state.lock().await;
        if let TunnelState::Connected(_)
        | TunnelState::Connecting(_)
        | TunnelState::Offline { reconnect: true }
        | TunnelState::Error(_) = app_state.tunnel
        {
            drop(app_state);
            let _ = vpnd.vpn_disconnect().await;
        }

        info!("app exit");
        app.exit(0);
    });
}
