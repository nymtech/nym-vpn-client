use tauri::State;
use tracing::{debug, instrument};

use crate::{
    error::{CmdError, CmdErrorSource},
    fs::data::{AppData, UiTheme},
    states::SharedAppData,
};

#[instrument(skip(shared_app_data))]
#[tauri::command]
pub async fn set_app_data(
    shared_app_data: State<'_, SharedAppData>,
    data: Option<AppData>,
) -> Result<(), CmdError> {
    debug!("set_app_data");
    let mut app_data_store = shared_app_data.lock().await;
    if let Some(data) = data {
        app_data_store.data = data;
    }
    app_data_store
        .write()
        .await
        .map_err(|e| CmdError::new(CmdErrorSource::InternalError, e.to_string()))?;

    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_app_data(
    shared_app_data: State<'_, SharedAppData>,
    data: Option<AppData>,
) -> Result<AppData, CmdError> {
    debug!("get_app_data");
    let mut app_data_store = shared_app_data.lock().await;
    if let Some(data) = data {
        app_data_store.data = data;
    }
    let data = app_data_store
        .read()
        .await
        .map_err(|e| CmdError::new(CmdErrorSource::InternalError, e.to_string()))?;

    Ok(data)
}

#[instrument(skip(shared_app_data))]
#[tauri::command]
pub async fn set_ui_theme(
    shared_app_data: State<'_, SharedAppData>,
    theme: UiTheme,
) -> Result<(), CmdError> {
    debug!("set_ui_theme");

    // save the selected UI theme to disk
    let mut app_data_store = shared_app_data.lock().await;
    let mut app_data = app_data_store
        .read()
        .await
        .map_err(|e| CmdError::new(CmdErrorSource::InternalError, e.to_string()))?;
    app_data.ui_theme = Some(theme);
    app_data_store.data = app_data;
    app_data_store
        .write()
        .await
        .map_err(|e| CmdError::new(CmdErrorSource::InternalError, e.to_string()))?;
    Ok(())
}

#[instrument(skip(shared_app_data))]
#[tauri::command]
pub async fn set_root_font_size(
    shared_app_data: State<'_, SharedAppData>,
    size: u32,
) -> Result<(), CmdError> {
    debug!("set_root_font_size");

    let mut app_data_store = shared_app_data.lock().await;
    let mut app_data = app_data_store
        .read()
        .await
        .map_err(|e| CmdError::new(CmdErrorSource::InternalError, e.to_string()))?;
    app_data.ui_root_font_size = Some(size);
    app_data_store.data = app_data;
    app_data_store
        .write()
        .await
        .map_err(|e| CmdError::new(CmdErrorSource::InternalError, e.to_string()))?;
    Ok(())
}

#[instrument(skip(shared_app_data))]
#[tauri::command]
pub async fn set_entry_location_selector(
    shared_app_data: State<'_, SharedAppData>,
    entry_selector: bool,
) -> Result<(), CmdError> {
    debug!("set_entry_location_selector");

    let mut app_data_store = shared_app_data.lock().await;
    let mut app_data = app_data_store
        .read()
        .await
        .map_err(|e| CmdError::new(CmdErrorSource::InternalError, e.to_string()))?;
    app_data.entry_location_selector = Some(entry_selector);
    app_data_store.data = app_data;
    app_data_store
        .write()
        .await
        .map_err(|e| CmdError::new(CmdErrorSource::InternalError, e.to_string()))?;
    Ok(())
}

#[instrument(skip(shared_app_data))]
#[tauri::command]
pub async fn set_auto_connect(
    shared_app_data: State<'_, SharedAppData>,
    entry_selector: bool,
) -> Result<(), CmdError> {
    debug!("set_auto_connect");

    let mut app_data_store = shared_app_data.lock().await;
    let mut app_data = app_data_store
        .read()
        .await
        .map_err(|e| CmdError::new(CmdErrorSource::InternalError, e.to_string()))?;
    app_data.autoconnect = Some(entry_selector);
    app_data_store.data = app_data;
    app_data_store
        .write()
        .await
        .map_err(|e| CmdError::new(CmdErrorSource::InternalError, e.to_string()))?;
    Ok(())
}

#[instrument(skip(shared_app_data))]
#[tauri::command]
pub async fn set_monitoring(
    shared_app_data: State<'_, SharedAppData>,
    entry_selector: bool,
) -> Result<(), CmdError> {
    debug!("set_monitoring");

    let mut app_data_store = shared_app_data.lock().await;
    let mut app_data = app_data_store
        .read()
        .await
        .map_err(|e| CmdError::new(CmdErrorSource::InternalError, e.to_string()))?;
    app_data.monitoring = Some(entry_selector);
    app_data_store.data = app_data;
    app_data_store
        .write()
        .await
        .map_err(|e| CmdError::new(CmdErrorSource::InternalError, e.to_string()))?;
    Ok(())
}
