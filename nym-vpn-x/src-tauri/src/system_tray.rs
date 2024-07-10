use std::sync::Arc;

use anyhow::Result;
use strum::AsRefStr;
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};
use tracing::{error, instrument, trace, warn};

use crate::{
    grpc::client::GrpcClient,
    states::{app::ConnectionState, SharedAppState},
    window::AppWindow,
    MAIN_WINDOW_LABEL,
};

#[derive(AsRefStr, Debug)]
enum TrayItemId {
    ShowHide,
    Quit,
}

pub fn systray(id: &str) -> SystemTray {
    let show = CustomMenuItem::new(TrayItemId::ShowHide.as_ref(), "Show/Hide");
    let quit = CustomMenuItem::new(TrayItemId::Quit.as_ref(), "Quit (disconnect)");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    SystemTray::new().with_id(id).with_menu(tray_menu)
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

#[instrument(skip_all)]
pub async fn on_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    let state = app.state::<SharedAppState>();
    let grpc = app.state::<Arc<GrpcClient>>();
    match event {
        SystemTrayEvent::LeftClick {
            position: _,
            size: _,
            ..
        } => {
            trace!("event left click");
            show_window(app, false).ok();
        }
        SystemTrayEvent::RightClick {
            position: _,
            size: _,
            ..
        } => {
            trace!("event right click");
        }
        SystemTrayEvent::DoubleClick {
            position: _,
            size: _,
            ..
        } => {
            trace!("event double click");
        }
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "Quit" => {
                trace!("event Quit");
                let app_state = state.lock().await;
                if let ConnectionState::Connected = app_state.state {
                    drop(app_state);
                    grpc.vpn_disconnect().await.ok();
                };
                app.exit(0);
            }
            "ShowHide" => {
                trace!("event ShowHide");
                show_window(app, true).ok();
            }
            _ => {}
        },
        _ => {}
    };
}
