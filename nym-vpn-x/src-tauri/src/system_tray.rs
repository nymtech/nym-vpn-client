use std::sync::Arc;

use strum::AsRefStr;
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};
use tracing::{debug, instrument, trace, warn};

use crate::{
    grpc::client::GrpcClient,
    states::{app::ConnectionState, SharedAppState},
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
                let window = app.get_window("main").unwrap_or_else(|| {
                    debug!("main window not found, re-creating it");
                    tauri::WindowBuilder::from_config(
                        app,
                        app.config().tauri.windows.first().unwrap().clone(),
                    )
                    .build()
                    .unwrap()
                });
                if window.is_visible().ok().unwrap_or(false) {
                    trace!("hiding main window");
                    window
                        .hide()
                        .inspect_err(|_| warn!("failed to hide main window"))
                        .ok();
                } else {
                    trace!("showing main window");
                    window
                        .show()
                        .inspect_err(|_| warn!("failed to show main window"))
                        .ok();
                    window
                        .set_focus()
                        .inspect_err(|_| warn!("failed to focus main window"))
                        .ok();
                }
            }
            _ => {}
        },
        _ => {}
    };
}

// &AppHandle<R>, tray::SystemTrayEvent
