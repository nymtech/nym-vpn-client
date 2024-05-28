use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, error, instrument};
use ts_rs::TS;

use crate::{
    country::Country,
    db::{Db, Key},
    error::{BackendError, ErrorKey},
    gateway::{get_gateway_countries, get_low_latency_entry_country},
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

#[instrument]
#[tauri::command]
pub async fn get_fastest_node_location() -> Result<Country, BackendError> {
    debug!("get_fastest_node_location");
    get_low_latency_entry_country().await.map_err(|e| {
        error!("failed to get fastest node location: {}", e);
        BackendError::new_internal("failed to get fastest node location", None)
    })
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

#[instrument]
#[tauri::command]
pub async fn get_countries(node_type: NodeType) -> Result<Vec<Country>, BackendError> {
    debug!("get_countries");
    match node_type {
        NodeType::Entry => get_gateway_countries(NodeType::Entry).await.map_err(|e| {
            BackendError::new_with_data(
                "failed to fetch entry countries",
                ErrorKey::GetEntryCountriesRequest,
                HashMap::from([("details".to_string(), e.to_string())]),
            )
        }),
        NodeType::Exit => get_gateway_countries(NodeType::Exit).await.map_err(|e| {
            BackendError::new_with_data(
                "failed to fetch exit countries",
                ErrorKey::GetExitCountriesRequest,
                HashMap::from([("details".to_string(), e.to_string())]),
            )
        }),
    }
}
