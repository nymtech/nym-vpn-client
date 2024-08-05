use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tracing::{debug, instrument};
use ts_rs::TS;

use crate::grpc::client::GrpcClient;
use crate::{
    country::Country,
    db::{Db, Key},
    error::{BackendError, ErrorKey},
    states::{app::NodeLocation, SharedAppState},
};

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
pub enum NodeType {
    Entry,
    Exit,
}

#[instrument(skip(app_state, db))]
#[tauri::command]
pub async fn set_node_location(
    app_state: State<'_, SharedAppState>,
    db: State<'_, Db>,
    node_type: NodeType,
    location: NodeLocation,
) -> Result<(), BackendError> {
    debug!("set_node_location");
    let mut state = app_state.lock().await;
    match node_type {
        NodeType::Entry => {
            state.entry_node_location = location.clone();
        }
        NodeType::Exit => {
            state.exit_node_location = location.clone();
        }
    }
    drop(state);

    debug!("saving new location in db");
    match node_type {
        NodeType::Entry => {
            db.insert(Key::EntryNodeLocation, &location)
                .map_err(|_| BackendError::new_internal("Failed to save location in db", None))?;
        }
        NodeType::Exit => {
            db.insert(Key::ExitNodeLocation, &location)
                .map_err(|_| BackendError::new_internal("Failed to save location in db", None))?;
        }
    }

    Ok(())
}

#[instrument(skip(app_state))]
#[tauri::command]
pub async fn get_node_location(
    app_state: State<'_, SharedAppState>,
    node_type: NodeType,
) -> Result<NodeLocation, BackendError> {
    debug!("get_node_location");
    Ok(match node_type {
        NodeType::Entry => app_state.lock().await.entry_node_location.clone(),
        NodeType::Exit => app_state.lock().await.exit_node_location.clone(),
    })
}

#[instrument(skip(grpc))]
#[tauri::command]
pub async fn get_countries(
    node_type: NodeType,
    grpc: State<'_, Arc<GrpcClient>>,
) -> Result<Vec<Country>, BackendError> {
    debug!("get_countries");
    match node_type {
        NodeType::Entry => grpc.entry_countries().await.map_err(|e| {
            BackendError::new_with_data(
                "failed to fetch entry countries",
                ErrorKey::GetEntryCountriesRequest,
                HashMap::from([("details", e.to_string())]),
            )
        }),
        NodeType::Exit => grpc.exit_countries().await.map_err(|e| {
            BackendError::new_with_data(
                "failed to fetch exit countries",
                ErrorKey::GetExitCountriesRequest,
                HashMap::from([("details", e.to_string())]),
            )
        }),
    }
}
