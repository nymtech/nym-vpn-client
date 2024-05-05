use std::sync::Arc;

use strum::AsRefStr;
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};
use tracing::{debug, instrument, trace};

use crate::{
    grpc::client::GrpcClient,
    states::{app::ConnectionState, SharedAppState},
};

#[derive(AsRefStr, Debug)]
enum TrayItemId {
    Show,
    Hide,
    Quit,
}

pub fn systray(id: &str) -> SystemTray {
    let show = CustomMenuItem::new(TrayItemId::Show.as_ref(), "Show");
    let hide = CustomMenuItem::new(TrayItemId::Hide.as_ref(), "Hide");
    let quit = CustomMenuItem::new(TrayItemId::Quit.as_ref(), "Quit (disconnect)");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(hide)
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
            "Hide" => {
                trace!("event Hide");
                if let Some(window) = app.get_window("main") {
                    window.hide().expect("failed to hide main window");
                }
            }
            "Show" => {
                trace!("event Show");
                let window = app.get_window("main").unwrap_or_else(|| {
                    debug!("main window not found, re-creating it");
                    tauri::WindowBuilder::from_config(
                        app,
                        app.config().tauri.windows.first().unwrap().clone(),
                    )
                    .build()
                    .unwrap()
                });
                window.show().expect("failed to show main window");
            }
            _ => {}
        },
        _ => {}
    };
}

// &AppHandle<R>, tray::SystemTrayEvent
