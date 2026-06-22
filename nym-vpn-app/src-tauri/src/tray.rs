use anyhow::Result;
#[cfg(not(target_os = "linux"))]
use strum::AsRefStr;
use tauri::{AppHandle, Manager};
use tracing::{info, instrument, trace, warn};

use crate::vpnd::tunnel::TunnelState;
use crate::{
    MAIN_WINDOW_LABEL, state::SharedAppState, vpnd::client::VpndClient, window::AppWindow,
};

#[cfg(not(target_os = "linux"))]
pub const TRAY_ICON_ID: &str = "main";

#[cfg(not(target_os = "linux"))]
#[derive(AsRefStr, Debug, Clone, Copy)]
enum MenuItemId {
    ShowHide,
    Quit,
    Status,
    Mode,
    Entry,
    Exit,
}

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

// ---------------------------------------------------------------------------
// Windows / macOS: native Tauri tray
// ---------------------------------------------------------------------------

#[cfg(not(target_os = "linux"))]
mod desktop {
    use std::time::Duration;

    use anyhow::Result;
    use tauri::{
        AppHandle, Manager, Wry,
        image::Image,
        include_image,
        menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
        tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    };
    use tokio::{sync::Mutex, task::JoinHandle};
    use tracing::{debug, error, instrument, trace, warn};

    use super::{MenuItemId, TRAY_ICON_ID, quit_app, show_window};
    use crate::APP_NAME;
    use crate::vpnd::tunnel::TunnelState;

    const APP_ICON: Image<'_> = include_image!("icons/tray_icon.png");
    const CONNECTED_ICON: Image<'_> = include_image!("icons/tray_icon_connected.png");
    const CONNECTING_ICON: Image<'_> = include_image!("icons/tray_icon_connecting.png");
    const DISCONNECTED_ICON: Image<'_> = include_image!("icons/tray_icon_disconnected.png");
    const ERROR_ICON: Image<'_> = include_image!("icons/tray_icon_error.png");
    const ICON_DEBOUNCE: Duration = Duration::from_millis(300);

    // Position of the entry item in the tray menu when visible:
    // show_hide(0), separator(1), status(2), mode(3), exit(4), entry(5), separator(6), quit(7)
    const ENTRY_MENU_POSITION: usize = 5;

    pub struct TrayManager {
        app: AppHandle,
        tray: TrayIcon,
        menu: Menu<Wry>,
        show_hide: MenuItem<Wry>,
        quit: MenuItem<Wry>,
        status: MenuItem<Wry>,
        mode: MenuItem<Wry>,
        entry: MenuItem<Wry>,
        exit: MenuItem<Wry>,
        entry_visible: Mutex<bool>,
        icon_debounce: Mutex<Option<JoinHandle<()>>>,
    }

    impl TrayManager {
        pub fn new(app: &AppHandle) -> Result<Self> {
            debug!("building system tray");

            // String labels are set in frontend (<TrayProvider>) to support localization
            let show_hide = MenuItem::with_id(
                app,
                MenuItemId::ShowHide.as_ref(),
                "Show/Hide",
                true,
                None::<&str>,
            )
            .inspect_err(|e| error!("failed to create menu item: {e}"))?;
            let quit = MenuItem::with_id(
                app,
                MenuItemId::Quit.as_ref(),
                "Quit (disconnect)",
                true,
                None::<&str>,
            )
            .inspect_err(|e| error!("failed to create menu item: {e}"))?;

            let status = MenuItem::with_id(
                app,
                MenuItemId::Status.as_ref(),
                "Status: Initial",
                true,
                None::<&str>,
            )
            .inspect_err(|e| error!("failed to create menu item: {e}"))?;
            let mode = MenuItem::with_id(
                app,
                MenuItemId::Mode.as_ref(),
                "Mode: Initial",
                true,
                None::<&str>,
            )
            .inspect_err(|e| error!("failed to create menu item: {e}"))?;
            let exit = MenuItem::with_id(
                app,
                MenuItemId::Exit.as_ref(),
                "Exit: Initial",
                true,
                None::<&str>,
            )
            .inspect_err(|e| error!("failed to create menu item: {e}"))?;
            let entry = MenuItem::with_id(
                app,
                MenuItemId::Entry.as_ref(),
                "Entry: Initial",
                true,
                None::<&str>,
            )
            .inspect_err(|e| error!("failed to create menu item: {e}"))?;

            let separator = PredefinedMenuItem::separator(app)?;

            let menu = Menu::with_items(
                app,
                &[
                    &show_hide, &separator, &status, &mode, &exit, &entry, &separator, &quit,
                ],
            )?;

            // Suppress the default "left-click opens menu" behavior so left-click can toggle
            // the window while right-click opens the context menu.
            let tray = TrayIconBuilder::with_id(TRAY_ICON_ID)
                .icon(APP_ICON)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(Self::on_tray_event)
                .on_menu_event(Self::on_menu_event)
                .build(app)?;

            let _ = tray
                .set_tooltip(Some(APP_NAME))
                .inspect_err(|e| error!("failed to set tray tooltip {e}"));

            Ok(Self {
                app: app.clone(),
                tray,
                menu,
                show_hide,
                quit,
                status,
                mode,
                entry,
                exit,
                entry_visible: Mutex::new(true),
                icon_debounce: Mutex::new(None),
            })
        }

        fn apply_icon(&self, state: &TunnelState) {
            let icon = match state {
                TunnelState::Connected(_) => CONNECTED_ICON,
                TunnelState::Connecting(_) | TunnelState::Disconnecting(_) => CONNECTING_ICON,
                TunnelState::Disconnected => DISCONNECTED_ICON,
                TunnelState::Error(_) | TunnelState::Offline { .. } => ERROR_ICON,
            };
            let _ = self.tray.set_icon(Some(icon));
        }

        #[instrument(skip_all)]
        pub async fn update_tray_icon(&self, state: TunnelState) {
            let mut pending = self.icon_debounce.lock().await;
            if let Some(handle) = pending.take() {
                handle.abort();
            }
            let app = self.app.clone();
            *pending = Some(tokio::spawn(async move {
                tokio::time::sleep(ICON_DEBOUNCE).await;
                let tray = app.state::<TrayManager>();
                tray.apply_icon(&state);
            }));
        }

        pub async fn update_tray_show_hide(&self, show_hide: String) {
            let _ = self.show_hide.set_text(show_hide);
        }

        pub async fn update_tray_quit(&self, quit: String) {
            let _ = self.quit.set_text(quit);
        }

        pub async fn update_tray_mode(&self, mode: String) {
            let _ = self.mode.set_text(mode);
        }

        pub async fn update_tray_state(&self, state: String) {
            let _ = self.status.set_text(state);
        }

        pub async fn update_tray_entry(&self, entry: String) {
            let _ = self.entry.set_text(entry);
        }

        pub async fn update_tray_exit(&self, exit: String) {
            let _ = self.exit.set_text(exit);
        }

        pub async fn update_tray_entry_visible(&self, visible: bool) {
            let mut current = self.entry_visible.lock().await;
            if *current == visible {
                return;
            }
            let result = if visible {
                self.menu.insert(&self.entry, ENTRY_MENU_POSITION)
            } else {
                self.menu.remove(&self.entry)
            };
            match result {
                Ok(()) => *current = visible,
                Err(e) => error!("failed to toggle tray entry visibility: {e}"),
            }
        }

        #[instrument(skip_all)]
        fn on_tray_event(tray_icon: &TrayIcon, event: TrayIconEvent) {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Down,
                ..
            } = event
            {
                trace!("tray event left click");
                let _ = show_window(tray_icon.app_handle(), true);
            }
        }

        #[instrument(skip(app))]
        fn on_menu_event(app: &AppHandle, event: MenuEvent) {
            trace!("menu event [{}]", event.id.0);

            match event.id().as_ref() {
                x if x == MenuItemId::ShowHide.as_ref() => {
                    trace!("show/hide menu clicked");
                    let _ = show_window(app, true);
                }
                x if x == MenuItemId::Quit.as_ref() => {
                    trace!("quit menu clicked");
                    quit_app(app.clone());
                }
                _ => warn!("unhandled menu event: {:?}", event.id),
            }
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub use desktop::TrayManager;

// ---------------------------------------------------------------------------
// Linux: ksni-based StatusNotifierItem
//
// We avoid Tauri's tray-icon backend on Linux because its GTK implementation only calls
// `app_indicator_set_menu()`. There's no API to receive left-click separately from the
// menu open, so we can't bind left-click to "toggle window". ksni implements SNI
// directly and exposes `activate` / `secondary_activate` callbacks.
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
mod linux {
    use std::sync::LazyLock;
    use std::time::Duration;

    use anyhow::Result;
    use image::ImageFormat;
    use ksni::{Handle, TrayMethods, menu::StandardItem};
    use tauri::AppHandle;
    use tokio::{sync::Mutex, task::JoinHandle};
    use tracing::{debug, instrument, trace, warn};

    use super::{quit_app, show_window};
    use crate::vpnd::tunnel::TunnelState;

    const APP_ICON: &[u8] = include_bytes!("../icons/tray_icon.png");
    const CONNECTED_ICON: &[u8] = include_bytes!("../icons/tray_icon_connected.png");
    const CONNECTING_ICON: &[u8] = include_bytes!("../icons/tray_icon_connecting.png");
    const DISCONNECTED_ICON: &[u8] = include_bytes!("../icons/tray_icon_disconnected.png");
    const ERROR_ICON: &[u8] = include_bytes!("../icons/tray_icon_error.png");
    const ICON_DEBOUNCE: Duration = Duration::from_millis(300);

    fn decode_argb(bytes: &[u8]) -> ksni::Icon {
        let img = image::load_from_memory_with_format(bytes, ImageFormat::Png)
            .expect("embedded tray icon must decode");
        let width = img.width() as i32;
        let height = img.height() as i32;
        let mut data = img.into_rgba8().into_vec();
        // ksni expects ARGB; image gives RGBA.
        for px in data.chunks_exact_mut(4) {
            px.rotate_right(1);
        }
        ksni::Icon {
            width,
            height,
            data,
        }
    }

    static APP_ARGB: LazyLock<ksni::Icon> = LazyLock::new(|| decode_argb(APP_ICON));
    static CONNECTED_ARGB: LazyLock<ksni::Icon> = LazyLock::new(|| decode_argb(CONNECTED_ICON));
    static CONNECTING_ARGB: LazyLock<ksni::Icon> = LazyLock::new(|| decode_argb(CONNECTING_ICON));
    static DISCONNECTED_ARGB: LazyLock<ksni::Icon> =
        LazyLock::new(|| decode_argb(DISCONNECTED_ICON));
    static ERROR_ARGB: LazyLock<ksni::Icon> = LazyLock::new(|| decode_argb(ERROR_ICON));

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum IconKind {
        Default,
        Connected,
        Connecting,
        Disconnected,
        Error,
    }

    impl IconKind {
        fn pixmap(self) -> ksni::Icon {
            match self {
                IconKind::Default => APP_ARGB.clone(),
                IconKind::Connected => CONNECTED_ARGB.clone(),
                IconKind::Connecting => CONNECTING_ARGB.clone(),
                IconKind::Disconnected => DISCONNECTED_ARGB.clone(),
                IconKind::Error => ERROR_ARGB.clone(),
            }
        }
    }

    impl From<&TunnelState> for IconKind {
        fn from(s: &TunnelState) -> Self {
            match s {
                TunnelState::Connected(_) => IconKind::Connected,
                TunnelState::Connecting(_) | TunnelState::Disconnecting(_) => IconKind::Connecting,
                TunnelState::Disconnected => IconKind::Disconnected,
                TunnelState::Error(_) | TunnelState::Offline { .. } => IconKind::Error,
            }
        }
    }

    struct NymTray {
        app: AppHandle,
        icon: IconKind,
        show_hide: String,
        quit: String,
        status: String,
        mode: String,
        entry: String,
        exit: String,
        entry_visible: bool,
    }

    impl ksni::Tray for NymTray {
        fn id(&self) -> String {
            // Stable per-app id so the panel can remember the slot.
            "nym-vpn-app".into()
        }

        fn title(&self) -> String {
            crate::APP_NAME.into()
        }

        fn tool_tip(&self) -> ksni::ToolTip {
            ksni::ToolTip {
                title: crate::APP_NAME.into(),
                description: self.status.clone(),
                ..Default::default()
            }
        }

        fn icon_pixmap(&self) -> Vec<ksni::Icon> {
            vec![self.icon.pixmap()]
        }

        fn activate(&mut self, _x: i32, _y: i32) {
            trace!("tray activate (left-click)");
            let _ = show_window(&self.app, true);
        }

        fn secondary_activate(&mut self, _x: i32, _y: i32) {
            trace!("tray secondary_activate (middle-click)");
            let _ = show_window(&self.app, true);
        }

        fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
            // Position parity with the desktop menu:
            // show_hide, separator, status, mode, exit, [entry], separator, quit
            let mut items: Vec<ksni::MenuItem<Self>> = Vec::with_capacity(8);

            items.push(
                StandardItem {
                    label: self.show_hide.clone(),
                    activate: Box::new(|t: &mut Self| {
                        trace!("show/hide menu clicked");
                        let _ = show_window(&t.app, true);
                    }),
                    ..Default::default()
                }
                .into(),
            );
            items.push(ksni::MenuItem::Separator);
            items.push(
                StandardItem {
                    label: self.status.clone(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
            items.push(
                StandardItem {
                    label: self.mode.clone(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
            items.push(
                StandardItem {
                    label: self.exit.clone(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
            if self.entry_visible {
                items.push(
                    StandardItem {
                        label: self.entry.clone(),
                        enabled: false,
                        ..Default::default()
                    }
                    .into(),
                );
            }
            items.push(ksni::MenuItem::Separator);
            items.push(
                StandardItem {
                    label: self.quit.clone(),
                    activate: Box::new(|t: &mut Self| {
                        trace!("quit menu clicked");
                        quit_app(t.app.clone());
                    }),
                    ..Default::default()
                }
                .into(),
            );

            items
        }
    }

    pub struct TrayManager {
        // `None` when the tray failed to spawn (e.g. no StatusNotifierWatcher on the bus,
        // as on bare i3/XEmbed setups). The app keeps running tray-less in that case;
        // all `update_tray_*` calls become no-ops.
        handle: Option<Handle<NymTray>>,
        icon_debounce: Mutex<Option<JoinHandle<()>>>,
        entry_visible: Mutex<bool>,
    }

    impl TrayManager {
        pub fn new(app: &AppHandle) -> Result<Self> {
            debug!("building ksni system tray");

            let tray = NymTray {
                app: app.clone(),
                icon: IconKind::Default,
                show_hide: "Show/Hide".into(),
                quit: "Quit (disconnect)".into(),
                status: "Status: Initial".into(),
                mode: "Mode: Initial".into(),
                entry: "Entry: Initial".into(),
                exit: "Exit: Initial".into(),
                entry_visible: true,
            };

            // We're called from Tauri's sync setup hook (which itself runs inside Tauri's
            // tokio runtime), so we can't `block_on` here. With the async-io feature, ksni
            // owns its own executor thread for the service task — we only need to drive
            // `spawn()` itself to completion. Do that on a dedicated thread with a plain
            // futures executor (no tokio).
            let (tx, rx) = std::sync::mpsc::sync_channel::<Result<Handle<NymTray>>>(1);
            std::thread::Builder::new()
                .name("ksni-tray-init".into())
                .spawn(move || {
                    let result = futures::executor::block_on(tray.spawn())
                        .map_err(|e| anyhow::anyhow!("failed to spawn ksni tray: {e:?}"));
                    let _ = tx.send(result);
                })?;

            // A failed spawn is non-fatal: SNI requires a StatusNotifierWatcher on the
            // session bus, which bare i3/XEmbed setups don't provide. Rather than crashing
            // the whole app out of Tauri's setup hook, log it and run without a tray.
            let handle = match rx.recv() {
                Ok(Ok(handle)) => Some(handle),
                Ok(Err(e)) => {
                    warn!("system tray unavailable, continuing without it: {e}");
                    None
                }
                Err(e) => {
                    warn!("ksni-tray init thread died, continuing without tray: {e}");
                    None
                }
            };

            Ok(Self {
                handle,
                icon_debounce: Mutex::new(None),
                entry_visible: Mutex::new(true),
            })
        }

        #[instrument(skip_all)]
        pub async fn update_tray_icon(&self, state: TunnelState) {
            let Some(handle) = self.handle.clone() else {
                return;
            };
            let mut pending = self.icon_debounce.lock().await;
            if let Some(h) = pending.take() {
                h.abort();
            }
            *pending = Some(tokio::spawn(async move {
                tokio::time::sleep(ICON_DEBOUNCE).await;
                let kind = IconKind::from(&state);
                handle.update(move |t: &mut NymTray| t.icon = kind).await;
            }));
        }

        pub async fn update_tray_show_hide(&self, show_hide: String) {
            let Some(handle) = &self.handle else { return };
            handle
                .update(move |t: &mut NymTray| t.show_hide = show_hide)
                .await;
        }

        pub async fn update_tray_quit(&self, quit: String) {
            let Some(handle) = &self.handle else { return };
            handle
                .update(move |t: &mut NymTray| t.quit = quit)
                .await;
        }

        pub async fn update_tray_mode(&self, mode: String) {
            let Some(handle) = &self.handle else { return };
            handle.update(move |t: &mut NymTray| t.mode = mode).await;
        }

        pub async fn update_tray_state(&self, state: String) {
            let Some(handle) = &self.handle else { return };
            handle
                .update(move |t: &mut NymTray| t.status = state)
                .await;
        }

        pub async fn update_tray_entry(&self, entry: String) {
            let Some(handle) = &self.handle else { return };
            handle
                .update(move |t: &mut NymTray| t.entry = entry)
                .await;
        }

        pub async fn update_tray_exit(&self, exit: String) {
            let Some(handle) = &self.handle else { return };
            handle.update(move |t: &mut NymTray| t.exit = exit).await;
        }

        pub async fn update_tray_entry_visible(&self, visible: bool) {
            let Some(handle) = &self.handle else { return };
            let mut current = self.entry_visible.lock().await;
            if *current == visible {
                return;
            }
            handle
                .update(move |t: &mut NymTray| t.entry_visible = visible)
                .await;
            *current = visible;
        }
    }
}

#[cfg(target_os = "linux")]
pub use linux::TrayManager;
