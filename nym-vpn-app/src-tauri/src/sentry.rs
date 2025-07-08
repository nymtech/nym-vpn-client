use crate::env::SENTRY_DSN;

use sentry::ClientInitGuard;
use std::time::Duration;
use tracing::{info, warn};

pub fn init() -> Option<ClientInitGuard> {
    let Some(dsn) = SENTRY_DSN.as_ref() else {
        warn!("failed to init sentry: SENTRY_DSN is not set");
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
    Some(guard)
}
