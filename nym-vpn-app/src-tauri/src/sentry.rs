use crate::env::APP_SENTRY_DSN;
use crate::sys::OsInfo;

use sentry::{ClientInitGuard, User};
use std::time::Duration;
use tracing::{info, warn};

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
