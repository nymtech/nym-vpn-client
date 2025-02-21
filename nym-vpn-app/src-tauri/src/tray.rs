use anyhow::Result;
use strum::AsRefStr;
use tauri::image::Image;
use tauri::menu::MenuEvent;
use tauri::tray::TrayIconEvent;
use tauri::tray::{MouseButton, MouseButtonState};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{include_image, Manager};
use tauri::{menu::MenuBuilder, AppHandle};
use tracing::{debug, error, info, instrument, trace, warn};

use crate::grpc::tunnel::TunnelState;
#[cfg(not(target_os = "linux"))]
use crate::APP_NAME;
use crate::{
    grpc::client::GrpcClient, states::SharedAppState, window::AppWindow, MAIN_WINDOW_LABEL,
};

pub const TRAY_ICON_ID: &str = "main";
pub const TRAY_MENU_ID: &str = "tray_menu";
const APP_ICON: Image<'_> = include_image!("icons/tray_icon.png");

#[derive(AsRefStr, Debug)]
enum MenuItemId {
    ShowHide,
    Quit,
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
        show_window(tray_icon.app_handle(), false).ok();
    }
}

#[instrument(skip(app))]
fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    trace!("menu event [{}]", event.id.0);

    match event.id().as_ref() {
        x if x == MenuItemId::ShowHide.as_ref() => {
            trace!("show/hide menu clicked");
            show_window(app, true).ok();
        }
        x if x == MenuItemId::Quit.as_ref() => {
            trace!("quit menu clicked");
            let c_app = app.clone();
            tokio::spawn(async move {
                let state = c_app.state::<SharedAppState>();
                let grpc = c_app.state::<GrpcClient>();

                let app_state = state.lock().await;
                if let TunnelState::Connected(_)
                | TunnelState::Connecting(_)
                | TunnelState::Offline { reconnect: true }
                | TunnelState::Error(_) = app_state.tunnel
                {
                    drop(app_state);
                    grpc.vpn_disconnect().await.ok();
                };
                info!("app exit");
                c_app.exit(0);
            });
        }
        _ => warn!("unhandled menu event: {:?}", event.id),
    }
}

#[instrument(skip_all)]
pub fn setup(app: &AppHandle) -> Result<()> {
    debug!("building system tray");
    let menu = MenuBuilder::with_id(app, TRAY_MENU_ID)
        .text(MenuItemId::ShowHide.as_ref(), "Show/Hide")
        .separator()
        .text(MenuItemId::Quit.as_ref(), "Quit (disconnect)")
        .build()
        .inspect_err(|e| error!("failed to build tray menu: {e}"))?;

    #[allow(unused_variables)]
    let tray = TrayIconBuilder::with_id(TRAY_ICON_ID)
        .icon(APP_ICON)
        .menu(&menu)
        .on_tray_icon_event(on_tray_event)
        .on_menu_event(on_menu_event)
        .build(app)?;

    #[cfg(not(target_os = "linux"))]
    tray.set_tooltip(Some(APP_NAME))
        .inspect_err(|e| error!("failed to set tray tooltip {e}"))
        .ok();

    Ok(())
}

fn show_window(app: &AppHandle, toggle: bool) -> Result<()> {
    let window = AppWindow::get_or_create(app, MAIN_WINDOW_LABEL)
        .inspect_err(|e| error!("failed to get main window {e}"))?;
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
