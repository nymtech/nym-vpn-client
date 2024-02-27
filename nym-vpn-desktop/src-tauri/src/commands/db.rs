use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, instrument, trace};
use ts_rs::TS;

use crate::{
    db::{Db, DbError, JsonValue, Key},
    error::{CmdError, CmdErrorSource},
    states::app::{NodeLocation, VpnMode},
};

#[derive(Default, Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub enum UiTheme {
    Dark,
    #[default]
    Light,
    System,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct AppData {
    pub monitoring: Option<bool>,
    pub autoconnect: Option<bool>,
    pub entry_location_enabled: Option<bool>,
    pub ui_theme: Option<UiTheme>,
    pub ui_root_font_size: Option<u32>,
    pub vpn_mode: Option<VpnMode>,
    pub entry_node_location: Option<NodeLocation>,
    pub exit_node_location: Option<NodeLocation>,
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn db_get_batch(db: State<'_, Db>) -> Result<AppData, CmdError> {
    debug!("db_get_batch");
    let err_msg = "Failed to retrieve db data".to_string();
    let entry_node_location = db
        .get_typed::<NodeLocation>(Key::EntryNodeLocation)
        .map_err(|_| CmdError::new(CmdErrorSource::InternalError, err_msg.clone()))?;
    let exit_node_location = db
        .get_typed::<NodeLocation>(Key::ExitNodeLocation)
        .map_err(|_| CmdError::new(CmdErrorSource::InternalError, err_msg.clone()))?;
    let vpn_mode = db
        .get_typed::<VpnMode>(Key::VpnMode)
        .map_err(|_| CmdError::new(CmdErrorSource::InternalError, err_msg.clone()))?;
    let ui_theme = db
        .get_typed::<UiTheme>(Key::UiTheme)
        .map_err(|_| CmdError::new(CmdErrorSource::InternalError, err_msg.clone()))?;
    let entry_location_enabled = db
        .get_typed::<bool>(Key::EntryLocationEnabled)
        .map_err(|_| CmdError::new(CmdErrorSource::InternalError, err_msg.clone()))?;
    let ui_root_font_size = db
        .get_typed::<u32>(Key::UiRootFontSize)
        .map_err(|_| CmdError::new(CmdErrorSource::InternalError, err_msg.clone()))?;
    let monitoring = db
        .get_typed::<bool>(Key::Monitoring)
        .map_err(|_| CmdError::new(CmdErrorSource::InternalError, err_msg.clone()))?;
    let autoconnect = db
        .get_typed::<bool>(Key::Autoconnect)
        .map_err(|_| CmdError::new(CmdErrorSource::InternalError, err_msg.clone()))?;

    trace!(
        r"
entry_node_location: {:?}
exit_node_location: {:?}
vpn_mode: {:?}
ui_theme: {:?}
entry_location_enabled: {:?}
ui_root_font_size: {:?}
monitoring: {:?}
autoconnect: {:?}",
        entry_node_location,
        exit_node_location,
        vpn_mode,
        ui_theme,
        entry_location_enabled,
        ui_root_font_size,
        monitoring,
        autoconnect
    );

    Ok(AppData {
        entry_node_location,
        exit_node_location,
        vpn_mode,
        ui_theme,
        entry_location_enabled,
        ui_root_font_size,
        monitoring,
        autoconnect,
    })
}

#[instrument(skip(db))]
#[tauri::command]
pub async fn db_get(db: State<'_, Db>, key: Key) -> Result<Option<JsonValue>, CmdError> {
    debug!("db_get");
    db.get(key).map_err(|_| {
        CmdError::new(
            CmdErrorSource::InternalError,
            format!("Failed to get key [{key}]"),
        )
    })
}

#[instrument(skip(db))]
#[tauri::command]
pub async fn db_set(
    db: State<'_, Db>,
    key: Key,
    value: JsonValue,
) -> Result<Option<JsonValue>, CmdError> {
    debug!("db_set");
    db.insert(key, &value).map_err(|e| match e {
        DbError::Serialize(e) => CmdError::new(
            CmdErrorSource::CallerError,
            format!("Failed to insert key, bad JSON input: {e}"),
        ),
        _ => CmdError::new(
            CmdErrorSource::InternalError,
            format!("Failed to insert key: {e}"),
        ),
    })
}

#[instrument(skip(db))]
#[tauri::command]
pub async fn db_flush(db: State<'_, Db>) -> Result<usize, CmdError> {
    debug!("db_flush");
    db.flush().await.map_err(|_| {
        CmdError::new(
            CmdErrorSource::InternalError,
            "Failed to flush db".to_string(),
        )
    })
}
