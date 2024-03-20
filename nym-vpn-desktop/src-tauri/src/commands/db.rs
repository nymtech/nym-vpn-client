use tauri::State;
use tracing::{debug, instrument};

use crate::{
    db::{Db, DbError, JsonValue, Key},
    error::{CmdError, CmdErrorSource},
};

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
