use sentry::{ClientInitGuard, Level, User};
use std::{
    borrow::Cow,
    collections::HashSet,
    sync::{Arc, OnceLock},
    time::Duration,
};
use tracing::{info, warn};

use crate::env::APP_SENTRY_DSN;
use crate::sys::OsInfo;

static EXCLUDED_ERRORS: OnceLock<HashSet<&'static str>> = OnceLock::new();

fn get_excluded_errors() -> &'static HashSet<&'static str> {
    EXCLUDED_ERRORS.get_or_init(|| {
        HashSet::from([
            "invalid mnemonic",
            "no device stored",
            "no account stored",
            "ac is offline",
            "account already exists",
            "maxdevicesreached",
            "subscriptionexpired",
        ])
    })
}

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
                    && get_excluded_errors()
                        .iter()
                        .any(|err| message.to_lowercase().contains(err))
                {
                    event.level = Level::Debug; // Change level to Debug
                }
                Some(event)
            })),
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
            id: Some(os.hash_identifier()), // anonymized user identifier
            ip_address: None,
            ..Default::default()
        }));
    });
    Some(guard)
}
