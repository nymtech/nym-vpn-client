use sentry::{ClientInitGuard, Level, User};
use std::{borrow::Cow, sync::Arc, time::Duration};
use tracing::{info, warn};

use crate::env::APP_SENTRY_DSN;
use crate::sys::OsInfo;

static EXCLUDED_ERRORS: [&str; 10] = [
    "failed to connect to the daemon: transport error",
    "vpnd down",
    "invalid passphrase",
    "no device stored",
    "no account stored",
    "ac is offline",
    "account already exists",
    "maxdevicesreached",
    "subscriptionexpired",
    // sled db open error when db is locked by another process
    // (another instance of the app)
    "IO error: could not acquire lock on",
];

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
            server_name: Some(Cow::Borrowed("nym")),
            before_send: Some(Arc::new(|mut event| {
                if matches!(event.level, Level::Error | Level::Warning)
                    && let Some(message) = &event.message
                    && EXCLUDED_ERRORS
                        .iter()
                        .any(|err| message.to_lowercase().contains(err))
                {
                    event.level = Level::Info;
                }
                Some(event)
            })),
            ..Default::default()
        },
    ));
    sentry::configure_scope(|scope| {
        scope.set_tag("os_version", &os.version);
        #[cfg(any(target_os = "linux", target_os = "openbsd"))]
        {
            if let Some(ds) = &os.display_server {
                scope.set_tag("display_server", ds.as_ref());
            }
            if let Some(gpu) = &os.gpu {
                scope.set_tag("gpu", gpu.as_ref());
            }
        }
        scope.set_user(Some(User {
            id: Some(os.hash.clone()), // anonymized user identifier
            ip_address: None,
            ..Default::default()
        }));
    });
    Some(guard)
}
