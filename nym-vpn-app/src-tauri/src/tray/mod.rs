#[cfg(target_os = "linux")]
use std::sync::Arc;
#[cfg(target_os = "linux")]
use std::time::Duration;

use anyhow::Result;
use tauri::{AppHandle, Manager};
#[cfg(target_os = "linux")]
use tokio::sync::Mutex;
use tracing::{info, instrument, trace, warn};

use crate::vpnd::tunnel::TunnelState;
use crate::{
    MAIN_WINDOW_LABEL, state::SharedAppState, vpnd::client::VpndClient, window::AppWindow,
};

// The native Tauri backend is compiled on every platform. On Linux it is the
// runtime fallback used when there is no StatusNotifierWatcher for ksni to talk to.
mod desktop;
#[cfg(target_os = "linux")]
mod linux;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IconKind {
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    Default,
    Connected,
    Connecting,
    Disconnected,
    Error,
}

impl From<&TunnelState> for IconKind {
    fn from(state: &TunnelState) -> Self {
        match state {
            TunnelState::Connected(_) => IconKind::Connected,
            TunnelState::Connecting(_) | TunnelState::Disconnecting(_) => IconKind::Connecting,
            TunnelState::Disconnected => IconKind::Disconnected,
            TunnelState::Error(_) | TunnelState::Offline { .. } => IconKind::Error,
        }
    }
}

/// Platform-agnostic contract that every tray backend implements.
///
/// The native Tauri backend ([`desktop`]) and the Linux ksni `StatusNotifierItem`
/// backend ([`linux`]) both implement this, so the compiler rejects any signature
/// drift between them. On Linux both are compiled and one is picked at runtime (see
/// the Linux [`Backend`] enum below); on Windows/macOS only [`desktop`] is used.
///
/// Construction is intentionally *not* part of this trait: each backend has its own
/// inherent constructor, because the Linux [`Backend`] enum decides which inner
/// backend to build only after probing the session bus for a StatusNotifierWatcher.
///
/// Note this trait is intentionally private: it is consumed only through the [`TrayManager`]
/// facade via static dispatch, so the async methods never need `Send` bounds added by callers.
trait TrayBackend: Send + Sync {
    async fn update_tray_icon(&self, icon: IconKind);
    async fn update_tray_show_hide(&self, show_hide: String);
    async fn update_tray_quit(&self, quit: String);
    async fn update_tray_mode(&self, mode: String);
    async fn update_tray_state(&self, state: String);
    async fn update_tray_entry(&self, entry: String);
    async fn update_tray_exit(&self, exit: String);
    async fn update_tray_entry_visible(&self, visible: bool);
}

#[cfg(not(target_os = "linux"))]
pub struct TrayManager(desktop::Backend);

#[cfg(not(target_os = "linux"))]
impl TrayManager {
    pub fn new(app: &AppHandle) -> Result<Self> {
        Ok(Self(desktop::Backend::new(app)?))
    }

    pub async fn update_tray_icon(&self, state: TunnelState) {
        self.0.update_tray_icon(IconKind::from(&state)).await
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

// ===========================================================================
// Linux: pick ksni when a StatusNotifierWatcher exists, else the native XEmbed
// backend; keep probing in the background and upgrade to ksni if a watcher
// appears within UPGRADE_DEADLINE.
// ===========================================================================

#[cfg(target_os = "linux")]
const UPGRADE_INTERVAL: Duration = Duration::from_millis(200);
#[cfg(target_os = "linux")]
const UPGRADE_DEADLINE: Duration = Duration::from_secs(120);

#[cfg(target_os = "linux")]
enum Backend {
    Ksni(linux::Backend),
    Native(Box<desktop::Backend>),
}

#[cfg(target_os = "linux")]
macro_rules! delegate_tray_backend {
    ($($method:ident($arg:ident: $arg_ty:ty)),+ $(,)?) => {
        $(
            async fn $method(&self, $arg: $arg_ty) {
                match self {
                    Self::Ksni(b) => b.$method($arg).await,
                    Self::Native(b) => b.$method($arg).await,
                }
            }
        )+
    };
}

#[cfg(target_os = "linux")]
impl TrayBackend for Backend {
    delegate_tray_backend! {
        update_tray_icon(icon: IconKind),
        update_tray_show_hide(show_hide: String),
        update_tray_quit(quit: String),
        update_tray_mode(mode: String),
        update_tray_state(state: String),
        update_tray_entry(entry: String),
        update_tray_exit(exit: String),
        update_tray_entry_visible(visible: bool),
    }
}

#[cfg(target_os = "linux")]
#[derive(Clone)]
struct TrayState {
    icon: IconKind,
    show_hide: String,
    quit: String,
    status: String,
    mode: String,
    entry: String,
    exit: String,
    entry_visible: bool,
}

#[cfg(target_os = "linux")]
impl Default for TrayState {
    fn default() -> Self {
        Self {
            icon: IconKind::Default,
            show_hide: "Show/Hide".into(),
            quit: "Quit (disconnect)".into(),
            status: "Status: Initial".into(),
            mode: "Mode: Initial".into(),
            entry: "Entry: Initial".into(),
            exit: "Exit: Initial".into(),
            entry_visible: true,
        }
    }
}

/// The system tray, managed as Tauri state and driven from `commands::tray`.
///
/// On Linux the backend can be swapped at runtime (native XEmbed -> ksni) by the
/// background upgrade task, so it lives behind a `Mutex`; `state` is the cache the
/// upgraded ksni tray is seeded from.
#[cfg(target_os = "linux")]
pub struct TrayManager {
    inner: Arc<Mutex<Backend>>,
    state: Arc<Mutex<TrayState>>,
}

#[cfg(target_os = "linux")]
impl TrayManager {
    pub fn new(app: &AppHandle) -> Result<Self> {
        let state = TrayState::default();

        // One instant ksni attempt, no retry sleeps: on a full desktop the
        // StatusNotifierWatcher is already up, so this succeeds immediately and we
        // never build the native tray or the upgrade task.
        let backend = match linux::spawn_ksni_once(app, &state) {
            Some(handle) => {
                info!("system tray: StatusNotifierWatcher found, using ksni (SNI) backend");
                Backend::Ksni(linux::Backend::from_handle(handle, state.entry_visible))
            }
            None => {
                warn!(
                    "system tray: no StatusNotifierWatcher on the session bus yet (e.g. i3); \
                     Will upgrade to ksni if a watcher appears within {}s.",
                    UPGRADE_DEADLINE.as_secs()
                );
                Backend::Native(Box::new(desktop::Backend::new(app)?))
            }
        };

        let needs_upgrade = matches!(backend, Backend::Native(_));
        let manager = Self {
            inner: Arc::new(Mutex::new(backend)),
            state: Arc::new(Mutex::new(state)),
        };
        if needs_upgrade {
            Self::spawn_upgrade_task(app.clone(), manager.inner.clone(), manager.state.clone());
        }
        Ok(manager)
    }

    // Retry `ksni::spawn()` (seeded from the live cache) until a watcher appears or
    // UPGRADE_DEADLINE elapses. On success, swap the native backend out for ksni,
    // drop the native tray on the main thread (its Drop touches GTK), and re-seed
    // the new tray from the latest cache.
    fn spawn_upgrade_task(
        app: AppHandle,
        inner: Arc<Mutex<Backend>>,
        state: Arc<Mutex<TrayState>>,
    ) {
        let spawned = std::thread::Builder::new()
            .name("ksni-tray-upgrade".into())
            .spawn(move || {
                let start = std::time::Instant::now();
                loop {
                    let snapshot = state.blocking_lock().clone();
                    if let Some(handle) = linux::spawn_ksni_once(&app, &snapshot) {
                        info!(
                            "system tray: StatusNotifierWatcher appeared, upgrading to ksni (SNI) backend"
                        );
                        let new_backend = Backend::Ksni(linux::Backend::from_handle(
                            handle,
                            snapshot.entry_visible,
                        ));
                        let old = {
                            let mut guard = inner.blocking_lock();
                            std::mem::replace(&mut *guard, new_backend)
                        };
                        let _ = app.run_on_main_thread(move || drop(old));
                        let inner = inner.clone();
                        let state = state.clone();
                        tauri::async_runtime::spawn(async move {
                            let guard = inner.lock().await;
                            let snapshot = state.lock().await.clone();
                            reseed(&guard, &snapshot).await;
                        });
                        return;
                    }
                    if start.elapsed() >= UPGRADE_DEADLINE {
                        warn!(
                            "system tray: no StatusNotifierWatcher appeared within {}s; \
                             staying on the native XEmbed tray for this session",
                            UPGRADE_DEADLINE.as_secs()
                        );
                        return;
                    }
                    std::thread::sleep(UPGRADE_INTERVAL);
                }
            });
        if let Err(e) = spawned {
            warn!("failed to start ksni upgrade probe thread: {e}");
        }
    }

    pub async fn update_tray_icon(&self, state: TunnelState) {
        let icon = IconKind::from(&state);
        self.state.lock().await.icon = icon;
        self.inner.lock().await.update_tray_icon(icon).await
    }

    pub async fn update_tray_show_hide(&self, show_hide: String) {
        self.state.lock().await.show_hide = show_hide.clone();
        self.inner
            .lock()
            .await
            .update_tray_show_hide(show_hide)
            .await
    }

    pub async fn update_tray_quit(&self, quit: String) {
        self.state.lock().await.quit = quit.clone();
        self.inner.lock().await.update_tray_quit(quit).await
    }

    pub async fn update_tray_mode(&self, mode: String) {
        self.state.lock().await.mode = mode.clone();
        self.inner.lock().await.update_tray_mode(mode).await
    }

    pub async fn update_tray_state(&self, state: String) {
        self.state.lock().await.status = state.clone();
        self.inner.lock().await.update_tray_state(state).await
    }

    pub async fn update_tray_entry(&self, entry: String) {
        self.state.lock().await.entry = entry.clone();
        self.inner.lock().await.update_tray_entry(entry).await
    }

    pub async fn update_tray_exit(&self, exit: String) {
        self.state.lock().await.exit = exit.clone();
        self.inner.lock().await.update_tray_exit(exit).await
    }

    pub async fn update_tray_entry_visible(&self, visible: bool) {
        {
            let mut state = self.state.lock().await;
            if state.entry_visible == visible {
                return;
            }
            state.entry_visible = visible;
        }
        self.inner
            .lock()
            .await
            .update_tray_entry_visible(visible)
            .await
    }
}

/// Push the full cache onto a backend (used to seed a freshly-upgraded ksni tray).
#[cfg(target_os = "linux")]
async fn reseed(backend: &Backend, state: &TrayState) {
    backend.update_tray_show_hide(state.show_hide.clone()).await;
    backend.update_tray_quit(state.quit.clone()).await;
    backend.update_tray_state(state.status.clone()).await;
    backend.update_tray_mode(state.mode.clone()).await;
    backend.update_tray_entry(state.entry.clone()).await;
    backend.update_tray_exit(state.exit.clone()).await;
    backend.update_tray_entry_visible(state.entry_visible).await;
    backend.update_tray_icon(state.icon).await;
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
