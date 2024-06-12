use std::sync::Arc;

use anyhow::{anyhow, Result};
use strum::AsRefStr;
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};
use tracing::{debug, instrument, trace, warn};

use crate::{
    grpc::client::GrpcClient,
    states::{app::ConnectionState, SharedAppState},
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
    let window = app
        .get_window(MAIN_WINDOW_LABEL)
        .or_else(|| {
            debug!("main window not found, re-creating it");
            tauri::WindowBuilder::from_config(
                app,
                app.config().tauri.windows.first().unwrap().clone(),
            )
            .build()
            .inspect_err(|e| warn!("failed to create main window: {e}"))
            .ok()
        })
        .ok_or(anyhow!("failed to get the main window"))?;
    let is_visible = window.is_visible().ok().unwrap_or(false);
    let is_minimized = window.is_minimized().unwrap_or(false);
    if !is_visible {
        trace!("showing main window");
        window
            .show()
            .inspect_err(|e| warn!("failed to show main window: {e}"))
            .ok();
        return window
            .set_focus()
            .inspect_err(|e| warn!("failed to focus main window: {e}"))
            .map_err(|e| e.into());
    }
    if is_visible && !is_minimized && toggle {
        trace!("hiding main window");
        return window
            .hide()
            .inspect_err(|e| warn!("failed to hide main window: {e}"))
            .map_err(|e| e.into());
    }

    if is_minimized {
        trace!("unminimizing main window");
        window
            .unminimize()
            .inspect_err(|e| warn!("failed to unminimize main window: {e}"))
            .ok();
        return window
            .set_focus()
            .inspect_err(|e| warn!("failed to focus main window: {e}"))
            .map_err(|e| e.into());
    }
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
