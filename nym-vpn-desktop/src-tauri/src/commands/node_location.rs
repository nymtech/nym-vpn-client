use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, error, instrument, trace};
use ts_rs::TS;

use crate::{
    country::{Country, FASTEST_NODE_LOCATION},
    error::{CmdError, CmdErrorSource},
    http::{
        client::HTTP_CLIENT,
        explorer_api::{JsonGateway, EXPLORER_API_URL, GATEWAYS_ENDPOINT},
    },
    states::{app::NodeLocation, SharedAppConfig, SharedAppData, SharedAppState},
};

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
pub enum NodeType {
    Entry,
    Exit,
}

#[instrument(skip(app_state, data_state))]
#[tauri::command]
pub async fn set_node_location(
    app_state: State<'_, SharedAppState>,
    data_state: State<'_, SharedAppData>,
    node_type: NodeType,
    location: NodeLocation,
) -> Result<(), CmdError> {
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

    // save the location on disk
    let mut app_data_store = data_state.lock().await;
    let mut app_data = app_data_store
        .read()
        .await
        .map_err(|e| CmdError::new(CmdErrorSource::InternalError, e.to_string()))?;

    match node_type {
        NodeType::Entry => {
            app_data.entry_node_location = Some(location);
        }
        NodeType::Exit => {
            app_data.exit_node_location = Some(location);
        }
    }
    app_data_store.data = app_data;
    app_data_store
        .write()
        .await
        .map_err(|e| CmdError::new(CmdErrorSource::InternalError, e.to_string()))?;

    Ok(())
}

#[instrument]
#[tauri::command]
pub async fn get_fastest_node_location() -> Result<Country, CmdError> {
    debug!("get_fastest_node_location");
    Ok(FASTEST_NODE_LOCATION.clone())
}

#[instrument(skip(app_state))]
#[tauri::command]
pub async fn get_node_location(
    app_state: State<'_, SharedAppState>,
    node_type: NodeType,
) -> Result<NodeLocation, CmdError> {
    debug!("get_node_location");
    Ok(match node_type {
        NodeType::Entry => app_state.lock().await.entry_node_location.clone(),
        NodeType::Exit => app_state.lock().await.exit_node_location.clone(),
    })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_node_countries(
    app_config: State<'_, SharedAppConfig>,
) -> Result<Vec<Country>, CmdError> {
    debug!("get_node_countries");
    let explorer_api = app_config
        .explorer_api_url
        .clone()
        .unwrap_or_else(|| EXPLORER_API_URL.to_string());
    let url = format!("{}/{}", explorer_api, GATEWAYS_ENDPOINT);

    debug!("fetching countries from explorer API [{url}]");
    let res = HTTP_CLIENT.get(url).send().await.map_err(|e| {
        error!("HTTP request GET /gateways failed: {e}");
        CmdError::new(
            CmdErrorSource::InternalError,
            "failed to fetch node locations".to_string(),
        )
    })?;

    debug!("deserializing json response");
    let json: Vec<JsonGateway> = res.json().await.map_err(|e| {
        error!("HTTP request GET /gateways failed to deserialize json response: {e}");
        CmdError::new(
            CmdErrorSource::InternalError,
            "failed to fetch node locations".to_string(),
        )
    })?;

    debug!("parsing json list");
    let list = json
        .into_iter()
        .filter_map(|gateway| gateway.location)
        .unique_by(|location| location.two_letter_iso_country_code.clone())
        .map(|location| {
            let mut name = location.country_name;
            // TODO yes this is what we get from the API for UK
            // let's use something more friendly
            if name == "United Kingdom of Great Britain and Northern Ireland" {
                name = "United Kingdom".to_string();
            }

            Country {
                name,
                code: location.two_letter_iso_country_code,
            }
        })
        // sort countries by name
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .collect::<Vec<_>>();

    debug!("fetched countries count [{}]", list.len());
    trace!("fetched countries {list:#?}");

    Ok(list)
}
