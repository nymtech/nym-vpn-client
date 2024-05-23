use tauri::State;
use tracing::{debug, instrument};

use crate::{
    db::{Db, DbError, JsonValue, Key},
    error::BkdError,
};

#[instrument(skip(db))]
#[tauri::command]
pub async fn db_get(db: State<'_, Db>, key: Key) -> Result<Option<JsonValue>, BkdError> {
    debug!("db_get");
    db.get(key)
        .map_err(|_| BkdError::new_internal(&format!("Failed to get key [{key}]"), None))
}

#[instrument(skip(db))]
#[tauri::command]
pub async fn db_set(
    db: State<'_, Db>,
    key: Key,
    value: JsonValue,
) -> Result<Option<JsonValue>, BkdError> {
    debug!("db_set");
    db.insert(key, &value).map_err(|e| match e {
        DbError::Serialize(e) => {
            BkdError::new_internal(&format!("Failed to insert key, bad JSON input: {e}"), None)
        }
        _ => BkdError::new_internal(&format!("Failed to insert key: {e}"), None),
    })
}

#[instrument(skip(db))]
#[tauri::command]
pub async fn db_flush(db: State<'_, Db>) -> Result<usize, BkdError> {
    debug!("db_flush");
    db.flush()
        .await
        .map_err(|_| BkdError::new_internal("Failed to flush db", None))
}
