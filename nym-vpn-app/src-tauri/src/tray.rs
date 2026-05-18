use std::time::Duration;

use anyhow::Result;
use strum::AsRefStr;
use tauri::{
    AppHandle, Manager, Wry,
    image::Image,
    include_image,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
};
use tokio::{sync::Mutex, task::JoinHandle};
use tracing::{debug, error, info, instrument, trace, warn};

#[cfg(not(target_os = "linux"))]
use crate::APP_NAME;
use crate::vpnd::tunnel::TunnelState;
use crate::{
    MAIN_WINDOW_LABEL, state::SharedAppState, vpnd::client::VpndClient, window::AppWindow,
};

pub const TRAY_ICON_ID: &str = "main";
const APP_ICON: Image<'_> = include_image!("icons/tray_icon.png");
const CONNECTED_ICON: Image<'_> = include_image!("icons/tray_icon_connected.png");
const CONNECTING_ICON: Image<'_> = include_image!("icons/tray_icon_connecting.png");
const DISCONNECTED_ICON: Image<'_> = include_image!("icons/tray_icon_disconnected.png");
const ERROR_ICON: Image<'_> = include_image!("icons/tray_icon_error.png");
const ICON_DEBOUNCE: Duration = Duration::from_millis(300);

#[derive(AsRefStr, Debug)]
enum MenuItemId {
    ShowHide,
    Quit,
    Status,
    Mode,
    Entry,
    Exit,
}

// Position of the entry item in the tray menu when visible:
// show_hide(0), separator(1), status(2), mode(3), entry(4), exit(5), separator(6), quit(7)
const ENTRY_MENU_POSITION: usize = 4;

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

        // String labels are set in frontent (<TrayProvider>) to support localization
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
        let entry = MenuItem::with_id(
            app,
            MenuItemId::Entry.as_ref(),
            "Entry: Initial",
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
        let separator = PredefinedMenuItem::separator(app)?;

        let menu = Menu::with_items(
            app,
            &[
                &show_hide, &separator, &status, &mode, &entry, &exit, &separator, &quit,
            ],
        )?;

        let tray = TrayIconBuilder::with_id(TRAY_ICON_ID)
            .icon(APP_ICON)
            .menu(&menu)
            .on_tray_icon_event(Self::on_tray_event)
            .on_menu_event(Self::on_menu_event)
            .build(app)?;

        #[cfg(not(target_os = "linux"))]
        tray.set_tooltip(Some(APP_NAME))
            .inspect_err(|e| error!("failed to set tray tooltip {e}"))
            .ok();

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
        self.tray.set_icon(Some(icon)).ok();
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
        self.show_hide.set_text(show_hide).ok();
    }

    pub async fn update_tray_quit(&self, quit: String) {
        self.quit.set_text(quit).ok();
    }

    pub async fn update_tray_mode(&self, mode: String) {
        self.mode.set_text(mode).ok();
    }

    pub async fn update_tray_state(&self, state: String) {
        self.status.set_text(state).ok();
    }

    pub async fn update_tray_entry(&self, entry: String) {
        self.entry.set_text(entry).ok();
    }

    pub async fn update_tray_exit(&self, exit: String) {
        self.exit.set_text(exit).ok();
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
            Self::show_window(tray_icon.app_handle(), false).ok();
        }
    }

    #[instrument(skip(app))]
    fn on_menu_event(app: &AppHandle, event: MenuEvent) {
        trace!("menu event [{}]", event.id.0);

        match event.id().as_ref() {
            x if x == MenuItemId::ShowHide.as_ref() => {
                trace!("show/hide menu clicked");
                Self::show_window(app, true).ok();
            }
            x if x == MenuItemId::Quit.as_ref() => {
                trace!("quit menu clicked");
                let c_app = app.clone();
                tokio::spawn(async move {
                    let state = c_app.state::<SharedAppState>();
                    let vpnd = c_app.state::<VpndClient>();

                    let app_state = state.lock().await;
                    if let TunnelState::Connected(_)
                    | TunnelState::Connecting(_)
                    | TunnelState::Offline { reconnect: true }
                    | TunnelState::Error(_) = app_state.tunnel
                    {
                        drop(app_state);
                        vpnd.vpn_disconnect().await.ok();
                    }

                    info!("app exit");
                    c_app.exit(0);
                });
            }
            _ => warn!("unhandled menu event: {:?}", event.id),
        }
    }

    #[instrument(skip(app))]
    fn show_window(app: &AppHandle, toggle: bool) -> Result<()> {
        let window = AppWindow::get_or_create(app, MAIN_WINDOW_LABEL)?;

        if !window.is_visible() {
            trace!("showing main window");
            window
                .0
                .show()
                .inspect_err(|e| warn!("failed to show main window: {e}"))
                .ok();
            window
                .0
                .set_focus()
                .inspect_err(|e| warn!("failed to focus main window: {e}"))
                .ok();
            return Ok(());
        }

        if window.is_visible() && !window.is_minimized() && toggle {
            trace!("hiding main window");
            window
                .0
                .hide()
                .inspect_err(|e| warn!("failed to hide main window: {e}"))
                .ok();
            return Ok(());
        }

        if window.is_minimized() {
            trace!("unminimizing main window");
            window
                .0
                .unminimize()
                .inspect_err(|e| warn!("failed to unminimize main window: {e}"))
                .ok();
            window
                .0
                .set_focus()
                .inspect_err(|e| warn!("failed to focus main window: {e}"))
                .ok();
            return Ok(());
        }

        window
            .0
            .set_focus()
            .inspect_err(|e| warn!("failed to focus main window: {e}"))
            .ok();

        Ok(())
    }
}
