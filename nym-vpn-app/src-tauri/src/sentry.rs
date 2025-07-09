use anyhow::Result;
use sentry::{ClientInitGuard, User};
use std::time::Duration;
use tracing::{error, info, instrument, warn};

use crate::env::APP_SENTRY_DSN;
use crate::grpc::client::GrpcClient;
use crate::sys::OsInfo;

#[instrument(skip_all)]
pub fn init(os: &OsInfo) -> Option<ClientInitGuard> {
    let Some(dsn) = APP_SENTRY_DSN.as_ref() else {
        warn!("failed to init sentry: APP_SENTRY_DSN is not set");
        return None;
    };
    info!("⚠ sentry monitoring enabled ⚠");
    let guard = sentry::init((
        dsn.to_owned(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            send_default_pii: false,
            sample_rate: 1.0,
            traces_sample_rate: 1.0,
            enable_logs: true,
            shutdown_timeout: Duration::from_secs(1),
            ..Default::default()
        },
    ));
    sentry::configure_scope(|scope| {
        scope.set_tag("os_version", &os.version);
        #[cfg(target_os = "linux")]
        {
            scope.set_tag("display_server", os.display_server.as_ref());
            scope.set_tag("gpu", os.gpu.as_ref());
        }
        scope.set_user(Some(User {
            ip_address: None,
            ..Default::default()
        }));
    });
    Some(guard)
}

// Check the state of sentry monitoring at daemon level and
// sync it with the app state if needed (as shown to the user in UI)
#[instrument(skip(grpc))]
pub async fn vpnd_check(sentry_enabled: bool, grpc: &GrpcClient) -> Result<()> {
    let vpnd_enabled = grpc.sentry_enabled().await.inspect_err(|e| {
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
        grpc.enable_sentry().await?;
    } else {
        info!("disabling vpnd sentry monitoring");
        grpc.disable_sentry().await?;
    }
    Ok(())
}
