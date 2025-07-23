use tracing::{error, info, instrument};

use crate::db::{Db, Key};
use crate::grpc::client::GrpcClient;

// Check the state of network statistics collection in daemon side
// if needed sync it with the saved setting from the app db
#[instrument(skip_all)]
pub async fn netstats_check(db: &Db, grpc: &GrpcClient) -> anyhow::Result<()> {
    let vpnd_enabled = grpc.netstats_enabled().await.inspect_err(|e| {
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
        grpc.enable_netstats().await?;
    } else {
        info!("disabled vpnd network statistics collection");
        grpc.disable_netstats().await?;
    }
    Ok(())
}
