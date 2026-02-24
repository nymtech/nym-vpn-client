use anyhow::Result;
use strum::AsRefStr;
use tauri::{
    image::Image,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle,
    Manager,
    Wry,
    include_image,
};
use tracing::{debug, error, info, instrument, trace, warn};

#[cfg(not(target_os = "linux"))]
use crate::APP_NAME;
use crate::vpnd::gateway::{EntryGatewayInfo, GatewayType};
use crate::vpnd::tunnel::{TunnelState, TunnelData};
use crate::{
    MAIN_WINDOW_LABEL, state::SharedAppState, vpnd::client::VpndClient,
    window::AppWindow,
};

pub const TRAY_ICON_ID: &str = "main";
const APP_ICON: Image<'_> = include_image!("icons/tray_icon.png");
const CONNECTED_ICON: Image<'_> = include_image!("icons/tray_icon_connected.png");
const CONNECTING_ICON: Image<'_> = include_image!("icons/tray_icon_connecting.png");
const DISCONNECTED_ICON: Image<'_> = include_image!("icons/tray_icon_disconnected.png");
const ERROR_ICON: Image<'_> = include_image!("icons/tray_icon_error.png");

#[derive(AsRefStr, Debug)]
enum MenuItemId {
    ShowHide,
    Quit,
    Status,
    Mode,
    Entry,
    Exit,
}

pub struct TrayManager {
    tray: TrayIcon,
    status: MenuItem<Wry>,
    mode: MenuItem<Wry>,
    entry: MenuItem<Wry>,
    exit: MenuItem<Wry>,
}

impl TrayManager {
    pub fn new(app: &AppHandle) -> Result<Self> {
        debug!("building system tray");


        let show_hide = MenuItem::with_id(app, MenuItemId::ShowHide.as_ref(), "Show/Hide", true, None::<&str>).inspect_err(|e| error!("failed to create menu item: {e}"))?;
        let quit = MenuItem::with_id(app, MenuItemId::Quit.as_ref(), "Quit (disconnect)", true, None::<&str>).inspect_err(|e| error!("failed to create menu item: {e}"))?;
        
        let status = MenuItem::with_id(app, MenuItemId::Status.as_ref(), "Status: Initial", true, None::<&str>).inspect_err(|e| error!("failed to create menu item: {e}"))?;
        let mode = MenuItem::with_id(app, MenuItemId::Mode.as_ref(), "Mode: Initial", true, None::<&str>).inspect_err(|e| error!("failed to create menu item: {e}"))?;
        let entry = MenuItem::with_id(app, MenuItemId::Entry.as_ref(), "Entry: Initial", true, None::<&str>).inspect_err(|e| error!("failed to create menu item: {e}"))?;
        let exit = MenuItem::with_id(app, MenuItemId::Exit.as_ref(), "Exit: Initial", true, None::<&str>).inspect_err(|e| error!("failed to create menu item: {e}"))?;
        let separator = PredefinedMenuItem::separator(app)?;

        let menu = Menu::with_items(app, &[&show_hide, &separator, &status, &mode, &entry, &exit, &separator, &quit])?;

        let tray = TrayIconBuilder::with_id(TRAY_ICON_ID)
            .icon(APP_ICON)
            .menu(&menu)
            .on_tray_icon_event(|tray, event| Self::on_tray_event(tray, event))
            .on_menu_event(|app, event| Self::on_menu_event(app, event))
            .build(app)?;

        #[cfg(not(target_os = "linux"))]
        tray.set_tooltip(Some(APP_NAME))
            .inspect_err(|e| error!("failed to set tray tooltip {e}"))
            .ok();
        
        Ok(Self {
            tray,
            status,
            mode,
            entry,
            exit,
        })
    }

    #[instrument(skip_all)]
    // pub fn update_tunnel(&self, state: TunnelState, entry_gw_info: Option<EntryGatewayInfo>) {
    pub async fn update_tunnel(&self, state: TunnelState, app: &AppHandle) {
        debug!("updating tunnel state: {:?}", state);
        match state {
            TunnelState::Connected(tunnel) => {
                self.tray.set_icon(Some(CONNECTED_ICON)).ok();

                let gw_type = match &tunnel.data {
                    TunnelData::Wireguard(_) => GatewayType::Wg,
                    TunnelData::Mixnet(_) => GatewayType::MxEntry,
                };
                
                let entry_gateway = app.state::<VpndClient>()
                    .gateways(gw_type)
                    .await
                    .ok()
                    .and_then(|gateways| {
                        gateways
                            .into_iter()
                            .find(|g| g.id == tunnel.entry_gw_id)
                            .map(|g| EntryGatewayInfo {
                                name: g.name,
                                country: g.country,
                                location: g.location,
                            })
                    });

                let exit_gateway = app.state::<VpndClient>()
                    .gateways(gw_type)
                    .await
                    .ok()
                    .and_then(|gateways| {
                        gateways
                            .into_iter()
                            .find(|g| g.id == tunnel.exit_gw_id)
                            .map(|g| EntryGatewayInfo {
                                name: g.name,
                                country: g.country,
                                location: g.location,
                            })
                    });

                let entry_gateway_display = entry_gateway
                    .as_ref()
                    .map(|g| g.to_string())
                    .unwrap_or_else(|| tunnel.entry_gw_id.clone());

                let exit_gateway_display = exit_gateway
                    .as_ref()
                    .map(|g| g.to_string())
                    .unwrap_or_else(|| tunnel.exit_gw_id.clone());


                 info!(
                    "Updated tray icon to connected, entry gateway: {}",
                    entry_gateway_display
                );
                info!(
                    "Updated tray icon to connected, exit gateway: {}",
                    exit_gateway_display
                );
                
                self.status.set_text("Status: Connected").ok(); 
                self.entry.set_text(&format!("Entry: {}", entry_gateway.unwrap().to_string())).ok();
                self.exit.set_text(&format!("Exit: {}", exit_gateway.unwrap().to_string())).ok();
                match gw_type {
                    GatewayType::MxEntry => {
                        self.mode.set_text("Mode: Anonymous(mixnet)").ok();
                    }
                    GatewayType::MxExit => {
                        self.mode.set_text("Mode: Anonymous(mixnet)").ok();
                    }
                    GatewayType::Wg => {
                        self.mode.set_text("Mode: Fast(WireGuard)").ok();
                    }
                }

                // let gateway_display = entry_gw_info
                //     .as_ref()
                //     .map(|g| g.to_string())
                //     .unwrap_or_else(|| tunnel.entry_gw_id.clone());
                // info!(
                //     "Updated tray icon to connected, entry gateway: {}",
                //     gateway_display
                // );
                // self.tray.set_tooltip(Some(format!("Connected to {}", tunnel.exit_gw.name)));
            }
            TunnelState::Connecting(connecting) => {
                self.tray.set_icon(Some(CONNECTING_ICON)).ok();
                self.status.set_text("Status: Connecting").ok();
                info!("Updated tray icon to connecting, connecting: {:?}", connecting);
                

                // let entry_gateway = app.state::<VpndClient>()
                //     .gateways(GatewayType::Wg)
                //     .await
                //     .ok()
                //     .and_then(|gateways| {
                //         gateways
                //             .into_iter()
                //             .find(|g| g.id == connecting.entry_gw_id)
                //             .map(|g| EntryGatewayInfo {
                //                 name: g.name,
                //                 country: g.country,
                //                 location: g.location,
                //             })
                //     });

                // let entry_gateway_display = entry_gateway
                //     .as_ref()
                //     .map(|g| g.to_string())
                //     .unwrap_or_else(|| connecting.entry_gw_id.clone().unwrap_or_else(|| "-".to_string()));

                // info!(
                //     "Updated tray icon to connecting, entry gateway: {}",
                //     entry_gateway_display
                // );

                // let exit_gateway = app.state::<VpndClient>()
                //     .gateways(GatewayType::Wg)
                //     .await
                //     .ok()
                //     .and_then(|gateways| {
                //         gateways
                //             .into_iter()
                //             .find(|g| g.id == connecting.exit_gw_id)
                //             .map(|g| EntryGatewayInfo {
                //                 name: g.name,
                //                 country: g.country,
                //                 location: g.location,
                //             })
                //     });

                // let exit_gateway_display = exit_gateway
                //     .as_ref()
                //     .map(|g| g.to_string())
                //     .unwrap_or_else(|| connecting.exit_gw_id.clone().unwrap_or_else(|| "-".to_string()));

                // info!(
                //     "Updated tray icon to connecting, exit gateway: {}",
                //     exit_gateway_display
                // );

                // self.entry.set_text(&format!("Entry: {}", entry_gateway.unwrap().to_string())).ok();
                // self.exit.set_text(&format!("Exit: {}", exit_gateway.unwrap().to_string())).ok();
            }
            TunnelState::Disconnected => {
                self.tray.set_icon(Some(DISCONNECTED_ICON)).ok();
                self.status.set_text("Status: Disconnected").ok();
                self.entry.set_text("Entry: -").ok();
                self.exit.set_text("Exit: -").ok();
                info!("Updated tray icon to disconnected");
            }
            TunnelState::Disconnecting(_) => {
                self.tray.set_icon(Some(CONNECTING_ICON)).ok();
                self.status.set_text("Status: Disconnecting").ok();
                info!("Updated tray icon to disconnecting");
            }
            TunnelState::Error(_) => {
                self.tray.set_icon(Some(ERROR_ICON)).ok();
                self.status.set_text("Status: Error").ok();
                self.entry.set_text("Entry: -").ok();
                self.exit.set_text("Exit: -").ok();
                info!("Updated tray icon to error");
            }
            TunnelState::Offline { reconnect: _ } => {
                self.tray.set_icon(Some(ERROR_ICON)).ok();
                self.status.set_text("Status: Offline").ok();
                self.entry.set_text("Entry: -").ok();
                self.exit.set_text("Exit: -").ok();
                info!("Updated tray icon to offline");
            }
            _ => {}
        }
    }

    pub async fn update_tray_mode(&self, mode: String) {
        self.mode.set_text(&format!("Mode: {}", mode)).ok();
    }

    pub async fn update_tray_state(&self, state: String) {
        self.status.set_text(state).ok();
        // self.status.set_text(&format!("{}", state)).ok();
    }

    pub async fn update_tray_entry(&self, entry: String) {
        info!("Updating tray entry: {}", entry);
        self.entry.set_text(entry).ok();
    }

    pub async fn update_tray_exit(&self, exit: String) {
        info!("Updating tray exit: {}", exit);
        self.exit.set_text(exit).ok();
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
