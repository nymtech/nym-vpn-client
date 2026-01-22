use crate::commands::gateway::Hop;
use crate::{
    error::{BackendError, ErrorKey},
    events::AppHandleEventEmitter,
    state::{SharedAppState, app::VpnMode},
    vpnd::{
        client::{Node, VpndClient, VpndError},
        config::VpndConfig,
        tunnel::{ConnectingState, TunnelState},
    },
};
use std::net::IpAddr;
use tauri::{Manager, State};
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
        info!("allow LAN: {}", config.allow_lan);
        info!("no IPv6: {}", config.disable_ipv6);
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
pub async fn set_enable_lewes_protocol(
    vpnd: State<'_, VpndClient>,
    enabled: bool,
) -> Result<(), BackendError> {
    vpnd.set_enable_lewes_protocol(enabled).await?;
    Ok(())
}
