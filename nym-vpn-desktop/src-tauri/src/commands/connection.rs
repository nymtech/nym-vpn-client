use futures::SinkExt;
use nym_vpn_lib::gateway_directory::{EntryPoint, ExitPoint};
use nym_vpn_lib::{NymVpnCtrlMessage, NymVpnHandle};
use std::env;
use tauri::State;
use tracing::{debug, error, info, instrument, trace};

use crate::country::FASTEST_NODE_LOCATION;
use crate::db::{Db, Key};
use crate::fs::path::BACKEND_DATA_PATH;
use crate::states::app::NodeLocation;
use crate::ENV_DISABLE_DATA_STORAGE;
use crate::{
    error::{CmdError, CmdErrorSource},
    events::{AppHandleEventEmitter, ConnectProgressMsg},
    states::{
        app::{ConnectionState, VpnMode},
        SharedAppState,
    },
    vpn_client::{create_vpn_config, spawn_exit_listener, spawn_status_listener},
};

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_connection_state(
    state: State<'_, SharedAppState>,
) -> Result<ConnectionState, CmdError> {
    debug!("get_connection_state");
    let app_state = state.lock().await;
    Ok(app_state.state.clone())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn connect(
    app: tauri::AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<ConnectionState, CmdError> {
    debug!("connect");
    {
        let mut app_state = state.lock().await;
        if app_state.state != ConnectionState::Disconnected {
            return Err(CmdError::new(
                CmdErrorSource::CallerError,
                &format!("cannot connect from state {:?}", app_state.state),
            ));
        };

        // switch to "Connecting" state
        trace!("update connection state [Connecting]");
        app_state.state = ConnectionState::Connecting;
    }
    app.emit_connecting();
    app.emit_connection_progress(ConnectProgressMsg::Initializing);

    let app_state = state.lock().await;

    let entry_point = match &app_state.entry_node_location {
        NodeLocation::Country(country) => {
            debug!("entry node location set, using: {}", country);
            EntryPoint::Location {
                location: country.code.clone(),
            }
        }
        NodeLocation::Fastest => {
            debug!(
                "entry node location set to `Fastest`, using: {}",
                FASTEST_NODE_LOCATION.clone()
            );
            EntryPoint::Location {
                location: FASTEST_NODE_LOCATION.code.clone(),
            }
        }
    };
    let exit_point = match &app_state.exit_node_location {
        NodeLocation::Country(country) => {
            debug!("exit node location set, using: {}", country);
            ExitPoint::Location {
                location: country.code.clone(),
            }
        }
        NodeLocation::Fastest => {
            debug!(
                "exit node location set to `Fastest`, using: {}",
                FASTEST_NODE_LOCATION.clone()
            );
            ExitPoint::Location {
                location: FASTEST_NODE_LOCATION.code.clone(),
            }
        }
    };

    let mut vpn_config = create_vpn_config(entry_point, exit_point);
    if let VpnMode::TwoHop = app_state.vpn_mode {
        info!("2-hop mode enabled");
        vpn_config.enable_two_hop = true;
    } else {
        info!("5-hop mode enabled");
    }
    vpn_config.enable_wireguard = false;
    // !! release app_state mutex
    // TODO: replace with automatic drop through scope
    drop(app_state);

    if !env::var(ENV_DISABLE_DATA_STORAGE).is_ok_and(|v| v == "true") {
        debug!(
            "using path for mixnet data: {}",
            BACKEND_DATA_PATH.to_string_lossy()
        );
        vpn_config.mixnet_data_path = Some(BACKEND_DATA_PATH.clone());
    }

    // spawn the VPN client and start a new connection
    let NymVpnHandle {
        vpn_ctrl_tx,
        vpn_status_rx,
        vpn_exit_rx,
    } = match nym_vpn_lib::spawn_nym_vpn_with_new_runtime(vpn_config).map_err(|e| {
        CmdError::new(
            CmdErrorSource::InternalError,
            &format!("fail to initialize Nym VPN client: {}", e),
        )
    }) {
        Ok(handle) => handle,
        Err(e) => {
            error!(e.message);
            trace!("update connection state [Disconnected]");
            let mut app_state = state.lock().await;
            app_state.state = ConnectionState::Disconnected;
            app.emit_disconnected(Some(e.message.clone()));
            return Err(e);
        }
    };
    info!("nym vpn client spawned");
    app.emit_connection_progress(ConnectProgressMsg::InitDone);

    // Start exit message listener
    // This will listen for the (single) exit message from the VPN client and update the UI accordingly
    debug!("starting exit listener");
    spawn_exit_listener(app.clone(), state.inner().clone(), vpn_exit_rx)
        .await
        .ok();

    // Start the VPN status listener
    // This will listen for status messages from the VPN client and update the UI accordingly
    debug!("starting status listener");
    spawn_status_listener(app, state.inner().clone(), vpn_status_rx)
        .await
        .ok();

    // Store the vpn control tx in the app state, which will be used to send control messages to
    // the running background VPN task, such as to disconnect.
    trace!("added vpn_ctrl_tx to app state");
    let mut state = state.lock().await;
    state.vpn_ctrl_tx = Some(vpn_ctrl_tx);

    Ok(state.state.clone())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn disconnect(
    app: tauri::AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<ConnectionState, CmdError> {
    debug!("disconnect");
    let mut app_state = state.lock().await;
    if !matches!(app_state.state, ConnectionState::Connected) {
        return Err(CmdError::new(
            CmdErrorSource::CallerError,
            &format!("cannot disconnect from state {:?}", app_state.state),
        ));
    };

    // switch to "Disconnecting" state
    trace!("update connection state [Disconnecting]");
    app_state.state = ConnectionState::Disconnecting;
    app.emit_disconnecting();

    let Some(ref mut vpn_tx) = app_state.vpn_ctrl_tx else {
        trace!("update connection state [Disconnected]");
        app_state.state = ConnectionState::Disconnected;
        app_state.connection_start_time = None;
        app.emit_disconnected(Some("vpn handle has not been initialized".to_string()));
        return Err(CmdError::new(
            CmdErrorSource::InternalError,
            "vpn handle has not been initialized",
        ));
    };

    // send Stop message to the VPN client
    debug!("sending Stop message to VPN client");
    vpn_tx.send(NymVpnCtrlMessage::Stop).await.map_err(|e| {
        let err_message = format!("failed to send Stop message to VPN client: {}", e);
        error!(err_message);
        app.emit_disconnected(Some(err_message.clone()));
        CmdError::new(CmdErrorSource::InternalError, &err_message)
    })?;
    debug!("Stop message sent");

    Ok(app_state.state.clone())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_connection_start_time(
    state: State<'_, SharedAppState>,
) -> Result<Option<i64>, CmdError> {
    debug!("get_connection_start_time");
    let app_state = state.lock().await;
    Ok(app_state.connection_start_time.map(|t| t.unix_timestamp()))
}

#[instrument(skip(app_state, db))]
#[tauri::command]
pub async fn set_vpn_mode(
    app_state: State<'_, SharedAppState>,
    db: State<'_, Db>,
    mode: VpnMode,
) -> Result<(), CmdError> {
    debug!("set_vpn_mode");

    let mut state = app_state.lock().await;

    if let ConnectionState::Disconnected = state.state {
    } else {
        let err_message = format!("cannot change vpn mode from state {:?}", state.state);
        error!(err_message);
        return Err(CmdError::new(CmdErrorSource::CallerError, &err_message));
    }
    state.vpn_mode = mode.clone();

    debug!("saving vpn mode in db");
    db.insert(Key::VpnMode, &mode).map_err(|_| {
        CmdError::new(
            CmdErrorSource::InternalError,
            "Failed to save vpn mode in db",
        )
    })?;
    Ok(())
}
