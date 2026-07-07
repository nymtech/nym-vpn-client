//! Native Tauri tray backend.
//!
//! Used on Windows/macOS, and on Linux as the runtime fallback when no
//! StatusNotifierWatcher is available for the ksni backend (see [`super::linux`]).

use std::time::Duration;

use anyhow::Result;
use strum::AsRefStr;
use tauri::{
    AppHandle, Wry,
    image::Image,
    include_image,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
};
use tokio::{sync::Mutex, task::JoinHandle};
use tracing::{debug, error, instrument, trace, warn};

use super::{IconKind, TrayBackend, quit_app, show_window};
use crate::APP_NAME;

const TRAY_ICON_ID: &str = "main";

const APP_ICON: Image<'_> = include_image!("icons/tray_icon.png");
const CONNECTED_ICON: Image<'_> = include_image!("icons/tray_icon_connected.png");
const CONNECTING_ICON: Image<'_> = include_image!("icons/tray_icon_connecting.png");
const DISCONNECTED_ICON: Image<'_> = include_image!("icons/tray_icon_disconnected.png");
const ERROR_ICON: Image<'_> = include_image!("icons/tray_icon_error.png");
const ICON_DEBOUNCE: Duration = Duration::from_millis(300);

// Position of the entry item in the tray menu when visible:
// show_hide(0), separator(1), status(2), mode(3), exit(4), entry(5), separator(6), quit(7)
const ENTRY_MENU_POSITION: usize = 5;

#[derive(AsRefStr, Debug, Clone, Copy)]
enum MenuItemId {
    ShowHide,
    Quit,
    Status,
    Mode,
    Entry,
    Exit,
}

pub(super) struct Backend {
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

fn image_for_kind(kind: IconKind) -> Image<'static> {
    match kind {
        IconKind::Default => APP_ICON,
        IconKind::Connected => CONNECTED_ICON,
        IconKind::Connecting => CONNECTING_ICON,
        IconKind::Disconnected => DISCONNECTED_ICON,
        IconKind::Error => ERROR_ICON,
    }
}

impl Backend {
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

    pub(super) fn new(app: &AppHandle) -> Result<Self> {
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

        let separator1 = PredefinedMenuItem::separator(app)?;
        let separator2 = PredefinedMenuItem::separator(app)?;

        let menu = Menu::with_items(
            app,
            &[
                &show_hide,
                &separator1,
                &status,
                &mode,
                &exit,
                &entry,
                &separator2,
                &quit,
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
}

impl TrayBackend for Backend {
    #[instrument(skip_all)]
    async fn update_tray_icon(&self, icon: IconKind) {
        let mut pending = self.icon_debounce.lock().await;
        if let Some(handle) = pending.take() {
            handle.abort();
        }
        // `TrayIcon` is reference-counted and `Clone`, so the debounced task can own a
        // clone and outlive this `&self` borrow without re-fetching from managed state.
        let tray = self.tray.clone();
        *pending = Some(tokio::spawn(async move {
            tokio::time::sleep(ICON_DEBOUNCE).await;
            let _ = tray.set_icon(Some(image_for_kind(icon)));
        }));
    }

    async fn update_tray_show_hide(&self, show_hide: String) {
        let _ = self.show_hide.set_text(show_hide);
    }

    async fn update_tray_quit(&self, quit: String) {
        let _ = self.quit.set_text(quit);
    }

    async fn update_tray_mode(&self, mode: String) {
        let _ = self.mode.set_text(mode);
    }

    async fn update_tray_state(&self, state: String) {
        let _ = self.status.set_text(state);
    }

    async fn update_tray_entry(&self, entry: String) {
        let _ = self.entry.set_text(entry);
    }

    async fn update_tray_exit(&self, exit: String) {
        let _ = self.exit.set_text(exit);
    }

    async fn update_tray_entry_visible(&self, visible: bool) {
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
}
