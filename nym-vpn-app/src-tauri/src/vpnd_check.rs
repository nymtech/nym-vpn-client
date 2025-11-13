use tracing::{error, info, instrument, warn};

use crate::db::{Db, Key};
use crate::vpnd::client::VpndClient;

// Check the state of network statistics collection in daemon side
// if needed sync it with the saved setting from the app db
#[instrument(skip_all)]
pub async fn netstats_check(db: &Db, vpnd: &VpndClient) -> anyhow::Result<()> {
    let vpnd_enabled = vpnd.netstats_enabled().await.inspect_err(|e| {
        error!("failed to check network stats collection: {:?}", e);
    })?;
    let app_enabled = db
        .get_typed::<bool>(Key::NetworkStatsEnabled.as_ref())
        .ok()
        .flatten()
        .unwrap_or(false);
    if vpnd_enabled == app_enabled {
        return Ok(());
    }
    info!(
        "network stats collection state mismatch: app: {}, vpnd: {}",
        app_enabled, vpnd_enabled
    );
    if app_enabled {
        info!("enabling vpnd network statistics collection");
        vpnd.enable_netstats().await?;
    } else {
        info!("disabled vpnd network statistics collection");
        vpnd.disable_netstats().await?;
    }
    Ok(())
}

// Check the state of sentry monitoring in daemon side
// if needed sync it with the saved setting from the app db
#[instrument(skip(vpnd))]
pub async fn sentry_check(sentry_enabled: bool, vpnd: &VpndClient) -> anyhow::Result<()> {
    let vpnd_enabled = vpnd.sentry_enabled().await.inspect_err(|e| {
        error!("failed to check sentry state: {:?}", e);
    })?;
    if vpnd_enabled == sentry_enabled {
        // all good
        return Ok(());
    }
    warn!(
        "sentry state mismatch: app sentry enabled: {}, vpnd sentry enabled: {}",
        sentry_enabled, vpnd_enabled
    );
    if sentry_enabled {
        info!("enabling vpnd sentry monitoring");
        vpnd.enable_sentry().await?;
    } else {
        info!("disabling vpnd sentry monitoring");
        vpnd.disable_sentry().await?;
    }
    Ok(())
}
