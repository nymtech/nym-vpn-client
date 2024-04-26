use nym_vpn_lib::gateway_directory::{Config, GatewayClient};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, instrument};
use ts_rs::TS;

use crate::{
    country::Country,
    db::{Db, Key},
    error::{CmdError, CmdErrorSource},
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

    debug!("saving new location in db");
    match node_type {
        NodeType::Entry => {
            db.insert(Key::EntryNodeLocation, &location).map_err(|_| {
                CmdError::new(
                    CmdErrorSource::InternalError,
                    "Failed to save location in db".to_string(),
                )
            })?;
        }
        NodeType::Exit => {
            db.insert(Key::ExitNodeLocation, &location).map_err(|_| {
                CmdError::new(
                    CmdErrorSource::InternalError,
                    "Failed to save location in db".to_string(),
                )
            })?;
        }
    }

    Ok(())
}

#[instrument]
#[tauri::command]
pub async fn get_fastest_node_location() -> Result<Country, CmdError> {
    debug!("get_fastest_node_location");
    get_low_latency_entry_country().await
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
        NodeType::Entry => get_gateway_countries(false).await,
        NodeType::Exit => get_gateway_countries(true).await,
    }
}

#[instrument(skip_all)]
async fn get_low_latency_entry_country() -> Result<Country, CmdError> {
    let config = Config::new_from_env();
    let gateway_client = GatewayClient::new(config)?;
    let described = gateway_client.lookup_low_latency_entry_gateway().await?;
    let country = described
        .location()
        .map(|l| Country {
            name: l.country_name.to_string(),
            code: l.two_letter_iso_country_code.to_string(),
        })
        .ok_or(CmdError {
            source: CmdErrorSource::InternalError,
            message: "Failed to get low latency country".to_string(),
        })?;

    Ok(country)
}

#[instrument(skip_all)]
async fn get_gateway_countries(exit_only: bool) -> Result<Vec<Country>, CmdError> {
    let config = Config::new_from_env();
    let gateway_client = GatewayClient::new(config)?;
    let locations = if !exit_only {
        gateway_client.lookup_all_countries().await?
    } else {
        gateway_client.lookup_all_exit_countries().await?
    };
    Ok(locations
        .into_iter()
        .map(|l| Country {
            name: l.country_name,
            code: l.two_letter_iso_country_code,
        })
        .collect())
}
