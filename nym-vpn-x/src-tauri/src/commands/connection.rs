use crate::country::FASTEST_NODE_LOCATION;
use crate::db::{Db, Key};
use crate::error::ErrorKey;
use crate::grpc::client::GrpcClient;
use crate::states::app::NodeLocation;
use crate::{
    error::BackendError,
    events::{AppHandleEventEmitter, ConnectProgressMsg},
    states::{
        app::{ConnectionState, VpnMode},
        SharedAppState,
    },
};
use nym_vpn_proto::entry_node::EntryNodeEnum;
use nym_vpn_proto::exit_node::ExitNodeEnum;
use nym_vpn_proto::{EntryNode, ExitNode, Location};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::{debug, error, info, instrument, trace};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConnectionStateResponse {
    state: ConnectionState,
    error: Option<BackendError>,
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_connection_state(
    state: State<'_, SharedAppState>,
    grpc: State<'_, Arc<GrpcClient>>,
) -> Result<ConnectionStateResponse, BackendError> {
    debug!("get_connection_state");
    let res = grpc.vpn_status().await?;
    let status = ConnectionState::from(res.status());
    let mut app_state = state.lock().await;
    app_state.state = status.clone();

    Ok(ConnectionStateResponse {
        state: status,
        error: res.error.map(BackendError::from),
    })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn connect(
    app: tauri::AppHandle,
    state: State<'_, SharedAppState>,
    grpc: State<'_, Arc<GrpcClient>>,
    entry: NodeLocation,
    exit: NodeLocation,
) -> Result<ConnectionState, BackendError> {
    debug!("connect");
    {
        let mut app_state = state.lock().await;
        if app_state.state != ConnectionState::Disconnected {
            return Err(BackendError::new_internal(
                &format!("cannot connect from state {:?}", app_state.state),
                None,
            ));
        };

        // switch to "Connecting" state
        trace!("update connection state [Connecting]");
        app_state.state = ConnectionState::Connecting;
    }

    app.emit_connecting();
    app.emit_connection_progress(ConnectProgressMsg::Initializing);

    let app_state = state.lock().await;
    let vpn_mode = app_state.vpn_mode.clone();
    let dns = app_state
        .dns_server
        .clone()
        .map(|ip| nym_vpn_proto::Dns { ip });
    // release the lock
    drop(app_state);

    let entry_node = match entry {
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

    let exit_node = match exit {
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

    let two_hop_mod = if let VpnMode::TwoHop = vpn_mode {
        info!("2-hop mode enabled");
        true
    } else {
        info!("5-hop mode enabled");
        false
    };

    app.emit_connection_progress(ConnectProgressMsg::InitDone);
    match grpc
        .vpn_connect(entry_node, exit_node, two_hop_mod, dns)
        .await
    {
        Ok(_) => Ok(ConnectionState::Connecting),
        Err(e) => {
            error!("grpc vpn_connect: {}", e);
            debug!("update connection state [Disconnected]");
            let mut app_state = state.lock().await;
            app_state.state = ConnectionState::Disconnected;
            drop(app_state);
            app.emit_disconnected(Some(BackendError::new(
                "Internal gRPC error",
                ErrorKey::GrpcError,
            )));
            Err(e.into())
        }
    }
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn disconnect(
    app: tauri::AppHandle,
    state: State<'_, SharedAppState>,
    grpc: State<'_, Arc<GrpcClient>>,
) -> Result<ConnectionState, BackendError> {
    debug!("disconnect");
    let mut app_state = state.lock().await;
    if !matches!(app_state.state, ConnectionState::Connected) {
        return Err(BackendError::new_internal(
            &format!("cannot disconnect from state {:?}", app_state.state),
            None,
        ));
    };
    app_state.state = ConnectionState::Disconnecting;
    drop(app_state);
    app.emit_disconnecting();

    grpc.vpn_disconnect().await?;
    Ok(ConnectionState::Disconnecting)
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_connection_start_time(
    state: State<'_, SharedAppState>,
) -> Result<Option<i64>, BackendError> {
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
) -> Result<(), BackendError> {
    debug!("set_vpn_mode");

    let mut state = app_state.lock().await;

    if let ConnectionState::Disconnected = state.state {
    } else {
        let err_message = format!("cannot change vpn mode from state {:?}", state.state);
        error!(err_message);
        return Err(BackendError::new_internal(&err_message, None));
    }
    state.vpn_mode = mode.clone();
    drop(state);

    debug!("saving vpn mode in db");
    db.insert(Key::VpnMode, &mode)
        .map_err(|_| BackendError::new_internal("Failed to save vpn mode in db", None))?;
    Ok(())
}
