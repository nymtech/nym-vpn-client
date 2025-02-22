use tauri::State;
use tracing::instrument;

use crate::{
    db::{Db, DbError, JsonValue},
    error::BackendError,
};

#[instrument(skip(db))]
#[tauri::command]
pub async fn db_get(db: State<'_, Db>, key: String) -> Result<Option<JsonValue>, BackendError> {
    db.get(&key)
        .map_err(|_| BackendError::internal(&format!("Failed to get key [{key}]"), None))
}

#[instrument(skip(db, value))]
#[tauri::command]
pub async fn db_set(
    db: State<'_, Db>,
    key: String,
    value: JsonValue,
) -> Result<Option<JsonValue>, BackendError> {
    db.insert(&key, &value).map_err(|e| match e {
        DbError::Serialize(e) => {
            BackendError::internal(&format!("Failed to insert key, bad JSON input: {e}"), None)
        }
        _ => BackendError::internal(&format!("Failed to insert key: {e}"), None),
    })
}

#[instrument(skip(db))]
#[tauri::command]
pub async fn db_del(db: State<'_, Db>, key: String) -> Result<Option<JsonValue>, BackendError> {
    db.remove(&key)
        .map_err(|_| BackendError::internal(&format!("Failed to remove key [{key}]"), None))
}

#[instrument(skip(db))]
#[tauri::command]
pub async fn db_flush(db: State<'_, Db>) -> Result<usize, BackendError> {
    db.flush()
        .await
        .map_err(|_| BackendError::internal("Failed to flush db", None))
}
