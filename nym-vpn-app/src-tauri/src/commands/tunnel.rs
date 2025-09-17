use crate::{
    db::{Db, Key},
    error::{BackendError, ErrorKey},
    events::AppHandleEventEmitter,
    grpc::{
        client::{GrpcClient, NodeConnect, VpndError},
        tunnel::{ConnectingState, TunnelState},
    },
    state::{SharedAppState, app::VpnMode},
};
use tauri::State;
use tracing::{debug, error, info, instrument, warn};

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_tunnel_state(
    app: tauri::AppHandle,
    grpc: State<'_, GrpcClient>,
) -> Result<TunnelState, BackendError> {
    let state = grpc.tunnel_state(&app).await?;
    Ok(state)
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn connect(
    app: tauri::AppHandle,
    state: State<'_, SharedAppState>,
    grpc: State<'_, GrpcClient>,
    db: State<'_, Db>,
    entry: NodeConnect,
    exit: NodeConnect,
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

    app.emit_connecting();
    let app_state = state.lock().await;
    let vpn_mode = app_state.vpn_mode.clone();

    let dns = app_state
        .dns_server
        .clone()
        .map(|ip| nym_vpn_proto::proto::Dns { ip: Some(ip) });
    let credentials_mode = app_state.credentials_mode;
    // release the lock
    drop(app_state);

    info!("entry {}", entry);
    info!("exit {}", exit);
    let two_hop_mod = if let VpnMode::Wg = vpn_mode {
        info!("mode [wg]");
        true
    } else {
        info!("mode [mixnet]");
        false
    };
    if credentials_mode {
        info!("credentials mode [on]");
    } else {
        info!("credentials mode [off]");
    }

    let use_netstack_wireguard = false;

    let disable_ipv6 = db
        .get_typed::<bool>(Key::DisableIpv6.as_ref())
        .unwrap_or(None)
        .unwrap_or(false);

    match grpc
        .vpn_connect(
            entry,
            exit,
            two_hop_mod,
            credentials_mode,
            use_netstack_wireguard,
            dns,
            disable_ipv6,
        )
        .await
    {
        Ok(_) => Ok(TunnelState::Connecting(ConnectingState::default())),
        Err(vpnd_err) => {
            warn!("grpc vpn_connect: {}", vpnd_err);
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
                        "Internal gRPC error",
                        ErrorKey::Grpc,
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
    grpc: State<'_, GrpcClient>,
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

    grpc.vpn_disconnect().await?;
    Ok(TunnelState::Disconnecting(None))
}

#[instrument(skip(app_state, db))]
#[tauri::command]
pub async fn set_vpn_mode(
    app_state: State<'_, SharedAppState>,
    db: State<'_, Db>,
    mode: VpnMode,
) -> Result<(), BackendError> {
    let mut state = app_state.lock().await;

    if matches!(
        state.tunnel,
        TunnelState::Connected(_) | TunnelState::Connecting(_) | TunnelState::Disconnecting(_)
    ) {
        let err_message = format!("cannot change vpn mode from state {}", state.tunnel);
        error!(err_message);
        return Err(BackendError::internal(&err_message, None));
    }
    state.vpn_mode = mode.clone();
    drop(state);

    db.insert(Key::VpnMode.as_ref(), &mode)
        .map_err(|_| BackendError::internal("Failed to save vpn mode in db", None))?;
    Ok(())
}
