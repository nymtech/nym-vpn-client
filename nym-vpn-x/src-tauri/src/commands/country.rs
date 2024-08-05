use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tracing::{debug, instrument};
use ts_rs::TS;

use crate::grpc::client::GrpcClient;
use crate::{
    country::Country,
    error::{BackendError, ErrorKey},
};

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
pub enum NodeType {
    Entry,
    Exit,
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
