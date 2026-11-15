use crate::db::{Db, Key};
use crate::error::BackendError;
use crate::vpnd::client::VpndClient;

use tauri::State;
use tracing::{error, instrument, warn};

#[instrument(skip_all)]
#[tauri::command]
pub async fn enable_netstats(
    db: State<'_, Db>,
    vpnd: State<'_, VpndClient>,
) -> Result<(), BackendError> {
    vpnd.enable_netstats().await?;
    if let Err(e) = db.insert(Key::NetworkStatsEnabled.as_ref(), true) {
        // vpnd already took the change, put it back so it matches the saved setting
        warn!("failed to save network stats setting: {e}, reverting vpnd");
        vpnd.disable_netstats()
            .await
            .inspect_err(|e| {
                error!("failed to revert vpnd network stats, it may still be enabled: {e}")
            })
            .ok();
        return Err(e.into());
    }
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn disable_netstats(
    db: State<'_, Db>,
    vpnd: State<'_, VpndClient>,
) -> Result<(), BackendError> {
    vpnd.disable_netstats().await?;
    if let Err(e) = db.insert(Key::NetworkStatsEnabled.as_ref(), false) {
        // vpnd already took the change, put it back so it matches the saved setting
        warn!("failed to save network stats setting: {e}, reverting vpnd");
        vpnd.enable_netstats()
            .await
            .inspect_err(|e| {
                error!("failed to revert vpnd network stats, it may still be disabled: {e}")
            })
            .ok();
        return Err(e.into());
    }
    Ok(())
}
