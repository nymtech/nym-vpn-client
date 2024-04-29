use crate::country::FASTEST_NODE_LOCATION;
use crate::db::{Db, Key};
use crate::grpc::client::GrpcClient;
use crate::states::app::NodeLocation;
use crate::vpnd;
use crate::{
    error::{CmdError, CmdErrorSource},
    events::{AppHandleEventEmitter, ConnectProgressMsg},
    states::{
        app::{ConnectionState, VpnMode},
        SharedAppState,
    },
};
use nym_vpn_proto::entry_node::EntryNodeEnum;
use nym_vpn_proto::exit_node::ExitNodeEnum;
use nym_vpn_proto::{
    ConnectRequest, DisconnectRequest, EntryNode, ExitNode, Location, StatusRequest,
};
use std::sync::Arc;
use tauri::State;
use tonic::Request;
use tracing::{debug, error, info, instrument, trace};

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_connection_state(
    state: State<'_, SharedAppState>,
    grpc_client: State<'_, Arc<GrpcClient>>,
) -> Result<ConnectionState, CmdError> {
    debug!("get_connection_state");

    let mut vpnd = grpc_client.vpnd().map_err(|_| {
        error!("not connected to nym daemon");
        CmdError::new(CmdErrorSource::DaemonError, "not connected to nym daemon")
    })?;

    let request = Request::new(StatusRequest {});
    let response = vpnd.vpn_status(request).await.map_err(|e| {
        error!("grpc vpn_status: {}", e);
        CmdError::new(
            CmdErrorSource::DaemonError,
            &format!("failed to get connection status: {e}"),
        )
    })?;
    debug!("grpc response: {:?}", response);

    let status = ConnectionState::from(response.into_inner().status());
    let mut app_state = state.lock().await;
    app_state.state = status.clone();
    Ok(status)
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn start_vpn_status_watchdog(
    app: tauri::AppHandle,
    grpc_client: State<'_, Arc<GrpcClient>>,
) -> Result<(), CmdError> {
    debug!("start_vpn_status_watchdog");

    vpnd::vpn_status_watchdog(&app, grpc_client.inner().as_ref())
        .await
        .map_err(|e| {
            error!("vpn_status_watchdog: {}", e);
            CmdError::new(
                CmdErrorSource::DaemonError,
                &format!("failed to start vpn status watchdog: {e}"),
            )
        })?;
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn connect(
    app: tauri::AppHandle,
    state: State<'_, SharedAppState>,
    grpc_client: State<'_, Arc<GrpcClient>>,
) -> Result<ConnectionState, CmdError> {
    debug!("connect");

    let mut vpnd = grpc_client.vpnd().map_err(|_| {
        error!("not connected to nym daemon");
        CmdError::new(CmdErrorSource::DaemonError, "not connected to nym daemon")
    })?;

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

    let mut app_state = state.lock().await;

    let entry_node = match &app_state.entry_node_location {
        NodeLocation::Country(country) => {
            debug!("entry node location set, using: {}", country);
            EntryNode {
                entry_node_enum: Some(EntryNodeEnum::Location(Location {
                    two_letter_iso_country_code: country.code.clone(),
                })),
            }
        }
        NodeLocation::Fastest => {
            debug!(
                "entry node location set to `Fastest`, using: {}",
                FASTEST_NODE_LOCATION.clone()
            );
            EntryNode {
                entry_node_enum: Some(EntryNodeEnum::Location(Location {
                    two_letter_iso_country_code: FASTEST_NODE_LOCATION.code.clone(),
                })),
            }
        }
    };

    let exit_node = match &app_state.exit_node_location {
        NodeLocation::Country(country) => {
            debug!("exit node location set, using: {}", country);
            ExitNode {
                exit_node_enum: Some(ExitNodeEnum::Location(Location {
                    two_letter_iso_country_code: country.code.clone(),
                })),
            }
        }
        NodeLocation::Fastest => {
            debug!(
                "exit node location set to `Fastest`, using: {}",
                FASTEST_NODE_LOCATION.clone()
            );
            ExitNode {
                exit_node_enum: Some(ExitNodeEnum::Location(Location {
                    two_letter_iso_country_code: FASTEST_NODE_LOCATION.code.clone(),
                })),
            }
        }
    };

    let two_hop_mod = if let VpnMode::TwoHop = app_state.vpn_mode {
        info!("2-hop mode enabled");
        true
    } else {
        info!("5-hop mode enabled");
        false
    };
    let request = Request::new(ConnectRequest {
        entry: Some(entry_node),
        exit: Some(exit_node),
        disable_routing: false,
        enable_two_hop: two_hop_mod,
        enable_poisson_rate: false,
        disable_background_cover_traffic: false,
        enable_credentials_mode: true,
        dns: app_state
            .dns_server
            .clone()
            .map(|ip| nym_vpn_proto::Dns { ip }),
    });

    app.emit_connection_progress(ConnectProgressMsg::InitDone);
    let response = vpnd.vpn_connect(request).await.map_err(|e| {
        let error_msg = format!("failed to connect: {e}");
        error!("grpc vpn_connect: {}", e);
        debug!("update connection state [Disconnected]");
        app_state.state = ConnectionState::Disconnected;
        app.emit_disconnected(Some(error_msg.clone()));
        CmdError::new(CmdErrorSource::DaemonError, &error_msg)
    })?;
    debug!("grpc response: {:?}", response);

    Ok(app_state.state.clone())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn disconnect(
    app: tauri::AppHandle,
    state: State<'_, SharedAppState>,
    grpc_client: State<'_, Arc<GrpcClient>>,
) -> Result<ConnectionState, CmdError> {
    debug!("disconnect");
    let mut app_state = state.lock().await;
    if !matches!(app_state.state, ConnectionState::Connected) {
        return Err(CmdError::new(
            CmdErrorSource::CallerError,
            &format!("cannot disconnect from state {:?}", app_state.state),
        ));
    };

    let mut vpnd = grpc_client.vpnd().map_err(|_| {
        error!("not connected to nym daemon");
        CmdError::new(CmdErrorSource::DaemonError, "not connected to nym daemon")
    })?;

    // switch to "Disconnecting" state
    trace!("update connection state [Disconnecting]");
    app_state.state = ConnectionState::Disconnecting;
    app.emit_disconnecting();

    let request = Request::new(DisconnectRequest {});
    let response = vpnd.vpn_disconnect(request).await.map_err(|e| {
        let error_msg = format!("failed to disconnect: {e}");
        error!("grpc vpn_disconnect: {}", e);
        // TODO handle error properly
        // just switch back to "Connected" state for now
        app_state.state = ConnectionState::Connected;
        CmdError::new(CmdErrorSource::DaemonError, &error_msg)
    })?;
    debug!("grpc response: {:?}", response);

    trace!("update connection state [Disconnected]");
    app_state.state = ConnectionState::Disconnected;
    app_state.connection_start_time = None;
    app.emit_disconnected(None);

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
