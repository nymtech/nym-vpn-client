use crate::db::{Db, Key};
use crate::error::BackendError;
use crate::vpnd::client::VpndClient;

use tauri::State;
use tracing::instrument;

#[instrument(skip_all)]
#[tauri::command]
pub async fn enable_netstats(
    db: State<'_, Db>,
    vpnd: State<'_, VpndClient>,
) -> Result<(), BackendError> {
    vpnd.enable_netstats().await?;
    db.insert(Key::NetworkStatsEnabled.as_ref(), true)?;
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn disable_netstats(
    db: State<'_, Db>,
    vpnd: State<'_, VpndClient>,
) -> Result<(), BackendError> {
    vpnd.disable_netstats().await?;
    db.insert(Key::NetworkStatsEnabled.as_ref(), false)?;
    Ok(())
}
