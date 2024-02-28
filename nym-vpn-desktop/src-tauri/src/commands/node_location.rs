use futures::future;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, error, instrument, trace};
use ts_rs::TS;

use crate::{
    country::{Country, FASTEST_NODE_LOCATION},
    error::{CmdError, CmdErrorSource},
    http::{
        client::{HttpError, HTTP_CLIENT},
        explorer_api, nym_api,
    },
    states::{app::NodeLocation, SharedAppData, SharedAppState},
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

#[instrument]
#[tauri::command]
pub async fn get_countries(node_type: NodeType) -> Result<Vec<Country>, CmdError> {
    debug!("get_countries");
    match node_type {
        NodeType::Entry => get_entry_countries().await,
        NodeType::Exit => get_exit_countries().await,
    }
}

#[instrument(skip_all)]
pub async fn get_entry_countries() -> Result<Vec<Country>, CmdError> {
    debug!("get_entry_countries");
    let response = explorer_api::get_gateways().await.map_err(|_| {
        CmdError::new(
            CmdErrorSource::InternalError,
            "failed to fetch node locations".to_string(),
        )
    })?;

    let json = explorer_api::deserialize_json(response)
        .await
        .map_err(|_| {
            CmdError::new(
                CmdErrorSource::InternalError,
                "failed to fetch node locations".to_string(),
            )
        })?;

    debug!("parsing json list");
    let list = json
        .into_iter()
        .filter_map(|gateway| gateway.location)
        // remove any duplicate two letter country code
        .unique_by(|location| location.two_letter_iso_country_code.clone())
        // mapping to a list of Country
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

#[instrument(skip_all)]
pub async fn get_exit_countries() -> Result<Vec<Country>, CmdError> {
    debug!("get_exit_countries");

    // future::join_all will collects the responses in the same
    // order
    let urls = vec![explorer_api::get_url(), nym_api::get_url()];

    debug!("fetching countries from Explorer and Nym api");
    // concurrently fetch both explorer and nym APIs
    let responses = future::join_all(urls.into_iter().map(|url| {
        let client = &HTTP_CLIENT;
        async move {
            client.get(&url).send().await.map_err(|e| {
                error!("HTTP request GET {url} failed: {e}");
                HttpError::RequestError(e.status())
            })
        }
    }))
    .await;
    trace!("fetching done");

    // filter out any failed requests
    let mut responses = responses
        .into_iter()
        .filter_map(|res| res.ok())
        .collect::<Vec<_>>();

    // if one of the requests failed, return an error
    if responses.len() != 2 {
        return Err(CmdError::new(
            CmdErrorSource::InternalError,
            "failed to fetch node locations".to_string(),
        ));
    }
    let explorer_response = responses.remove(0);
    let nym_api_response = responses.remove(0);

    debug!("deserializing json responses");
    let explorer_json = explorer_api::deserialize_json(explorer_response)
        .await
        .map_err(|_| {
            CmdError::new(
                CmdErrorSource::InternalError,
                "failed to fetch node locations".to_string(),
            )
        })?;

    let nym_api_json = nym_api::deserialize_json(nym_api_response)
        .await
        .map_err(|_| {
            CmdError::new(
                CmdErrorSource::InternalError,
                "failed to fetch node locations".to_string(),
            )
        })?;

    debug!("parsing responses");
    let list = nym_api_json
        .into_iter()
        // only retain gateways with IP packet router
        // mapping to a list of identity keys
        .filter_map(|gateway| {
            gateway
                .self_described
                .and_then(|desc| desc.ip_packet_router)
                .map(|_| gateway.bond.gateway().identity_key.clone())
        })
        // remove any duplicate identity key
        .unique()
        // find the corresponding country code in the explorer response
        // for each identity key
        // mapping to a list of locations
        .filter_map(|identity_key| {
            explorer_json.iter().find_map(|gateway| {
                if gateway.gateway.identity_key == identity_key {
                    gateway.location.clone()
                } else {
                    None
                }
            })
        })
        // remove any duplicate two letter country code
        .unique_by(|location| location.two_letter_iso_country_code.clone())
        // mapping to a list of Country
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
