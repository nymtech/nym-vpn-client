use crate::commands::gateway::Hop;
use crate::{
    db::Db,
    error::{BackendError, ErrorKey},
    events::AppHandleEventEmitter,
    fs::app_discovery::{App, custom_apps, get_installed_apps},
    state::{SharedAppState, app::VpnMode},
    vpnd::{
        client::{Node, VpndClient, VpndError},
        config::{MixnetTrafficConfig, MixnetTrafficDefaults, VpndConfig},
        tunnel::{ConnectingState, FrontingMode, SplitApp, TunnelState},
    },
};
use std::net::IpAddr;
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;
use tracing::{debug, info, instrument, warn};

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_tunnel_state(
    app: tauri::AppHandle,
    vpnd: State<'_, VpndClient>,
) -> Result<TunnelState, BackendError> {
    let state = vpnd.tunnel_state(&app).await?;
    Ok(state)
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_vpn_config(app: tauri::AppHandle) -> Result<Option<VpndConfig>, BackendError> {
    let s_state = app.state::<SharedAppState>();
    let app_state = s_state.lock().await;
    Ok(app_state.vpnd_config.clone())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn connect(
    app: tauri::AppHandle,
    state: State<'_, SharedAppState>,
    vpnd: State<'_, VpndClient>,
) -> Result<TunnelState, BackendError> {
    {
        let mut app_state = state.lock().await;
        if app_state.tunnel != TunnelState::Disconnected {
            let msg = format!("cannot connect from state {:?}", app_state.tunnel);
            warn!(msg);
            return Err(BackendError::internal(&msg, None));
        };

        // manually switch to "Connecting" state
        debug!("update connection state [Connecting]");
        app_state.tunnel = TunnelState::Connecting(ConnectingState::default());
    }
    let app_state = state.lock().await;
    if let Some(config) = &app_state.vpnd_config {
        info!("vpn mode: {}", config.vpn_mode);
        info!("entry node: {}", config.entry_node);
        info!("exit node: {}", config.exit_node);
        info!("QUIC mode: {}", config.bridges);
        info!("fronting mode: {}", config.fronting_mode.to_string());
        info!("allow LAN: {}", config.allow_lan);
        info!("no IPv6: {}", config.disable_ipv6);
        info!("mixnet traffic config: {:?}", config.mixnet_traffic);
    } else {
        warn!("no vpnd config available");
    }

    app.emit_connecting();

    match vpnd.vpn_connect().await {
        Ok(_) => Ok(TunnelState::Connecting(ConnectingState::default())),
        Err(vpnd_err) => {
            warn!("vpn_connect: {}", vpnd_err);
            debug!("update connection state [Disconnected]");
            let mut app_state = state.lock().await;
            app_state.tunnel = TunnelState::Disconnected;
            drop(app_state);
            match vpnd_err {
                VpndError::Response(ref e) => {
                    app.emit_disconnected(Some(e.clone()));
                }
                _ => {
                    app.emit_disconnected(Some(BackendError::new(
                        "Internal rpc client error",
                        ErrorKey::VpndClient,
                    )));
                }
            }
            Err(vpnd_err.into())
        }
    }
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn disconnect(
    app: tauri::AppHandle,
    state: State<'_, SharedAppState>,
    vpnd: State<'_, VpndClient>,
) -> Result<TunnelState, BackendError> {
    let mut app_state = state.lock().await;
    if matches!(
        app_state.tunnel,
        TunnelState::Disconnected | TunnelState::Disconnecting(_)
    ) {
        let msg = format!("cannot connect from state {:?}", app_state.tunnel);
        warn!(msg);
        return Err(BackendError::internal(&msg, None));
    };
    app_state.tunnel = TunnelState::Disconnecting(None);
    debug!("update connection state [Disconnecting]");
    drop(app_state);
    app.emit_disconnecting();

    vpnd.vpn_disconnect().await?;
    Ok(TunnelState::Disconnecting(None))
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn reconnect(vpnd: State<'_, VpndClient>) -> Result<(), BackendError> {
    vpnd.vpn_reconnect().await?;
    Ok(())
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_vpn_mode(vpnd: State<'_, VpndClient>, mode: VpnMode) -> Result<(), BackendError> {
    vpnd.set_two_hop(mode == VpnMode::Wg).await?;
    Ok(())
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_node(
    vpnd: State<'_, VpndClient>,
    node: Node,
    hop: Hop,
) -> Result<(), BackendError> {
    match hop {
        Hop::Entry => vpnd.set_entry_node(node).await?,
        Hop::Exit => vpnd.set_exit_node(node).await?,
    }
    Ok(())
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_quic(vpnd: State<'_, VpndClient>, enabled: bool) -> Result<(), BackendError> {
    vpnd.set_quic(enabled).await?;
    Ok(())
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_fronting_mode(
    vpnd: State<'_, VpndClient>,
    mode: FrontingMode,
) -> Result<(), BackendError> {
    vpnd.set_fronting_mode(mode).await?;
    Ok(())
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_no_ipv6(vpnd: State<'_, VpndClient>, enabled: bool) -> Result<(), BackendError> {
    vpnd.set_no_ipv6(enabled).await?;
    Ok(())
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_allow_lan(vpnd: State<'_, VpndClient>, enabled: bool) -> Result<(), BackendError> {
    vpnd.set_allow_lan(enabled).await?;
    Ok(())
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_ad_block(vpnd: State<'_, VpndClient>, enabled: bool) -> Result<(), BackendError> {
    vpnd.set_ad_block(enabled).await?;
    Ok(())
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn get_default_dns(vpnd: State<'_, VpndClient>) -> Result<Vec<IpAddr>, BackendError> {
    let dns = vpnd.get_default_dns().await?;
    Ok(dns)
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_custom_dns_enabled(
    vpnd: State<'_, VpndClient>,
    enabled: bool,
) -> Result<(), BackendError> {
    vpnd.set_custom_dns_enabled(enabled).await?;
    Ok(())
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_custom_dns(
    vpnd: State<'_, VpndClient>,
    dns: Vec<IpAddr>,
) -> Result<(), BackendError> {
    vpnd.set_custom_dns(dns).await?;
    Ok(())
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn get_privy_derivation_message(
    vpnd: State<'_, VpndClient>,
) -> Result<String, BackendError> {
    let message = vpnd.get_privy_derivation_message().await?;
    Ok(message)
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_mixnet_traffic_config(
    vpnd: State<'_, VpndClient>,
    config: MixnetTrafficConfig,
) -> Result<(), BackendError> {
    vpnd.set_mixnet_traffic_config(config).await?;
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn calculate_traffic_latency(config: MixnetTrafficConfig) -> Result<f64, BackendError> {
    let lib_config: nym_vpn_lib_types::MixnetTrafficConfig = config.into();
    Ok(lib_config.calculate_traffic_latency())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_mixnet_traffic_defaults() -> Result<MixnetTrafficDefaults, BackendError> {
    Ok(MixnetTrafficDefaults::get())
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_enable_split_tunnel(
    vpnd: State<'_, VpndClient>,
    enabled: bool,
) -> Result<(), BackendError> {
    vpnd.enable_split_tunnel(enabled).await?;
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_app_list(
    app: tauri::AppHandle,
    db: State<'_, Db>,
) -> Result<Vec<App>, BackendError> {
    let app_handle = app.clone();
    let discovered = tokio::task::spawn_blocking(move || get_installed_apps(app_handle))
        .await
        .map_err(|e| BackendError::internal(&e.to_string(), None))??;
    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut custom = custom_apps::load(&db)?;

    #[cfg(windows)]
    {
        use std::collections::HashSet;
        use std::path::Path;

        use crate::state::SharedAppState;
        use tauri::Manager as _;

        let daemon_paths: Vec<String> = {
            let s_state = app.state::<SharedAppState>();
            let state = s_state.lock().await;
            state
                .vpnd_config
                .as_ref()
                .map(|c| c.split_tunnel.apps.iter().map(|a| a.path.clone()).collect())
                .unwrap_or_default()
        };

        if !daemon_paths.is_empty() {
            let discovered_set: HashSet<String> = discovered
                .iter()
                .map(|a| a.executable_path.to_ascii_lowercase())
                .collect();
            let custom_set: HashSet<String> = custom
                .iter()
                .map(|a| a.executable_path.to_ascii_lowercase())
                .collect();

            let paths_to_import: Vec<String> = daemon_paths
                .into_iter()
                .filter(|p| {
                    let key = p.to_ascii_lowercase();
                    !discovered_set.contains(&key) && !custom_set.contains(&key)
                })
                .collect();

            if !paths_to_import.is_empty() {
                let app_clone = app.clone();
                let new_apps: Vec<App> = tokio::task::spawn_blocking(move || {
                    paths_to_import
                        .iter()
                        .filter_map(|path| {
                            match custom_apps::build_custom_app(Path::new(path), Some(&app_clone)) {
                                Ok(new_app) => {
                                    info!(
                                        "importing daemon split-tunnel app into custom list: {path}"
                                    );
                                    Some(new_app)
                                }
                                Err(e) => {
                                    warn!("skipping daemon split-tunnel app '{path}': {e}");
                                    None
                                }
                            }
                        })
                        .collect()
                })
                .await
                .map_err(|e| BackendError::internal(&e.to_string(), None))?;

                if !new_apps.is_empty() {
                    custom.extend(new_apps);
                    custom_apps::save(&db, &custom)?;
                }
            }
        }
    }

    Ok(custom_apps::merge(discovered, custom))
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn add_app_to_split_tunnel(
    vpnd: State<'_, VpndClient>,
    app: SplitApp,
) -> Result<(), BackendError> {
    info!("[command] add_app_to_split_tunnel: {}", app.path);
    vpnd.add_app_to_split_tunnel(app).await?;
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn remove_app_from_split_tunnel(
    vpnd: State<'_, VpndClient>,
    app: SplitApp,
) -> Result<(), BackendError> {
    vpnd.remove_app_from_split_tunnel(app).await?;
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn is_split_tunnel_supported(vpnd: State<'_, VpndClient>) -> Result<bool, BackendError> {
    let is_supported = vpnd.is_split_tunnel_supported().await?;
    Ok(is_supported)
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn add_custom_split_tunnel_app(
    app: tauri::AppHandle,
    db: State<'_, Db>,
) -> Result<Option<App>, BackendError> {
    let app_clone = app.clone();
    let file_path = tokio::task::spawn_blocking(move || {
        let picker = app_clone.dialog().file();
        #[cfg(windows)]
        let picker = picker.add_filter("Executable files", &["exe"]);
        picker.blocking_pick_file()
    })
    .await
    .map_err(|e| BackendError::internal(&e.to_string(), None))?;
    let Some(file_path) = file_path else {
        info!("user cancelled custom split tunnel app dialog");
        return Ok(None);
    };

    let path = file_path
        .as_path()
        .ok_or_else(|| BackendError::internal("failed to resolve picked file path", None))?
        .to_path_buf();
    info!("[command] add_custom_split_tunnel_app: {}", path.display());

    let new_app = custom_apps::build_custom_app(&path, Some(&app))?;
    let mut apps = custom_apps::load(&db)?;
    custom_apps::insert_unique(&mut apps, new_app.clone())?;
    custom_apps::save(&db, &apps)?;

    Ok(Some(new_app))
}

#[instrument(skip_all)]
#[tauri::command]
#[cfg_attr(not(windows), allow(unused_variables))]
pub async fn remove_custom_split_tunnel_app(
    app: tauri::AppHandle,
    vpnd: State<'_, VpndClient>,
    db: State<'_, Db>,
    path: String,
) -> Result<(), BackendError> {
    info!("[command] remove_custom_split_tunnel_app: {path}");
    #[cfg(windows)]
    {
        let is_in_daemon_list = {
            let s_state = app.state::<SharedAppState>();
            let state = s_state.lock().await;
            state
                .vpnd_config
                .as_ref()
                .map(|c| {
                    c.split_tunnel
                        .apps
                        .iter()
                        .any(|a| a.path.eq_ignore_ascii_case(&path))
                })
                .unwrap_or(false)
        };
        if is_in_daemon_list {
            vpnd.remove_app_from_split_tunnel(SplitApp { path: path.clone() })
                .await?;
        }
    }
    let mut apps = custom_apps::load(&db)?;
    custom_apps::remove(&mut apps, &path);
    custom_apps::save(&db, &apps)?;
    Ok(())
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_enable_geo_location(
    vpnd: State<'_, VpndClient>,
    enabled: bool,
) -> Result<(), BackendError> {
    vpnd.set_enable_geo_location(enabled).await?;
    Ok(())
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_geo_exclusion_enabled(
    vpnd: State<'_, VpndClient>,
    enabled: bool,
) -> Result<(), BackendError> {
    vpnd.set_geo_exclusion_enabled(enabled).await?;
    Ok(())
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_geo_exclusion_listen_port(
    vpnd: State<'_, VpndClient>,
    port: u16,
) -> Result<(), BackendError> {
    vpnd.set_geo_exclusion_listen_port(port).await?;
    Ok(())
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_geo_exclusion_excluded_countries(
    vpnd: State<'_, VpndClient>,
    countries: Vec<String>,
) -> Result<(), BackendError> {
    vpnd.set_geo_exclusion_excluded_countries(countries).await?;
    Ok(())
}
