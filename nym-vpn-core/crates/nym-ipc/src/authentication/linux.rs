// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{ffi::CString, time::Duration};

use nix::{
    sys::socket::{UnixCredentials, getsockopt, sockopt::PeerCredentials},
    unistd::{Gid, Uid, User, getgrouplist},
};
use tokio::io::AsyncWriteExt;
use tokio_stream::Stream;
use tokio_util::sync::CancellationToken;
use zbus::Connection;
use zbus_polkit::policykit1::{
    AuthorityProxy, AuthorizationResult, CheckAuthorizationFlags, Subject,
};

use crate::{
    AuthenticationMaterial,
    authentication::{AuthenticationLayer, error::AuthenticationError},
    uds::Uds,
};

pub(crate) type Transport = tokio::net::UnixStream;

const ACTION_ID: &str = "com.nymvpn.vpnd.unix-access";
const CANCELLATION_ID: &str = "com.nymvpn.vpnd.cancel";
const USER_INTERACTION_TIMEOUT: Duration = Duration::from_secs(60);

// Single source of truth for the polkit action XML.
const POLKIT_POLICY: &str = include_str!("../../.pkg/com.nymvpn.vpnd.unix-access.policy");

// Polkit's documented action search path is /usr/share/polkit-1/actions/.
// Most distros also scan /etc/polkit-1/actions/, which we use as a fallback
// when /usr/share is not writable (e.g. Fedora Silverblue, MicroOS, NixOS,
// other ostree-based images) so a daemon installed by hand can still register
// its action without root having to relayer the OS image.
const POLICY_PRIMARY_DIR: &str = "/usr/share/polkit-1/actions";
const POLICY_FALLBACK_DIRS: &[&str] = &["/etc/polkit-1/actions"];

#[async_trait::async_trait]
trait AuthorizationChecker {
    async fn check_authorization(&self) -> Result<AuthorizationResult, AuthenticationError>;
    async fn cancel_check_authorization(&self) -> Result<(), AuthenticationError>;
}

#[async_trait::async_trait]
trait Prompter {
    async fn prompt_for_authorization(
        &self,
        cred: UnixCredentials,
    ) -> Result<AuthorizationResult, AuthenticationError>;
}

struct AuthProxy<'a> {
    pub proxy: AuthorityProxy<'a>,
    pub subject: Subject,
}

#[async_trait::async_trait]
impl AuthorizationChecker for AuthProxy<'_> {
    async fn check_authorization(&self) -> Result<AuthorizationResult, AuthenticationError> {
        let already_registered = self
            .proxy
            .enumerate_actions("")
            .await
            .map_err(AuthenticationError::EnumerateActions)?
            .iter()
            .any(|action| action.action_id == ACTION_ID);

        if already_registered {
            tracing::debug!(
                action = ACTION_ID,
                "polkit action already registered; skipping install",
            );
        } else {
            install_polkit_policy(POLICY_PRIMARY_DIR, POLICY_FALLBACK_DIRS).await?;
        }

        // details might be useful to set some locale-sensitive messages and icon images in the authentication dialog
        let details = std::collections::HashMap::new();
        self.proxy
            .check_authorization(
                &self.subject,
                ACTION_ID,
                &details,
                CheckAuthorizationFlags::AllowUserInteraction.into(),
                CANCELLATION_ID,
            )
            .await
            .map_err(AuthenticationError::CheckAuthorization)
    }
    async fn cancel_check_authorization(&self) -> Result<(), AuthenticationError> {
        self.proxy
            .cancel_check_authorization(CANCELLATION_ID)
            .await
            .map_err(AuthenticationError::CancelAuthorization)
    }
}

// Try each candidate directory in order and return the first one we managed
// to write into. A successful install at /usr/share is the architecturally
// correct outcome (polkit's documented path); /etc is the defensive fallback
// for read-only-/usr systems.
async fn install_polkit_policy(
    primary_dir: &str,
    fallback_dirs: &[&str],
) -> Result<String, AuthenticationError> {
    let path = format!("{primary_dir}/{ACTION_ID}.policy");
    match write_policy_file(&path).await {
        Ok(()) => {
            tracing::info!(
                path = %path,
                "installed polkit action policy",
            );
            Ok(path)
        }
        Err(primary_err) => {
            // try fallback directories in case the primary one doesn't work
            for dir in fallback_dirs {
                let path = format!("{dir}/{ACTION_ID}.policy");
                match write_policy_file(&path).await {
                    Ok(()) => {
                        tracing::info!(
                            path = %path,
                            "installed polkit action policy",
                        );
                        return Ok(path);
                    }
                    Err(fallback_err) => {
                        tracing::debug!(
                            path = %path,
                            error = %fallback_err,
                            "polkit policy create failed, trying next candidate",
                        );
                    }
                }
            }
            tracing::error!(
                error = %primary_err,
                primary_candidate = primary_dir,
                fallback_candidates = ?fallback_dirs,
                "failed to install polkit policy in any candidate directory; \
                 see the daemon log file at /var/log/nym-vpnd/nym-vpnd.log \
                 and the troubleshooting docs",
            );
            Err(primary_err)
        }
    }
}

async fn write_policy_file(path: &str) -> Result<(), AuthenticationError> {
    // Some distros (notably Fedora Silverblue) ship `/etc/polkit-1/rules.d/`
    // but not `/etc/polkit-1/actions/` because no package installs there by
    // default. Create the parent on demand so the fallback path works on a
    // stock install. This is a no-op when the dir already exists.
    if let Some(parent) = std::path::Path::new(path).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(AuthenticationError::CreateActionPolicy)?;
    }

    let mut file = tokio::fs::File::create(path)
        .await
        .map_err(AuthenticationError::CreateActionPolicy)?;
    file.write_all(POLKIT_POLICY.as_bytes())
        .await
        .map_err(AuthenticationError::WriteActionPolicy)?;
    file.flush()
        .await
        .map_err(AuthenticationError::WriteActionPolicy)?;
    Ok(())
}

struct PolkitPrompter {
    shutdown_token: CancellationToken,
}

impl PolkitPrompter {
    fn new(shutdown_token: CancellationToken) -> Self {
        Self { shutdown_token }
    }
}

#[async_trait::async_trait]
impl Prompter for PolkitPrompter {
    async fn prompt_for_authorization(
        &self,
        cred: UnixCredentials,
    ) -> Result<AuthorizationResult, AuthenticationError> {
        let connection = self
            .shutdown_token
            .run_until_cancelled(Connection::system())
            .await
            .ok_or(AuthenticationError::ShuttingDown)?
            .map_err(AuthenticationError::MessageBusConnection)?;
        let proxy = self
            .shutdown_token
            .run_until_cancelled(AuthorityProxy::new(&connection))
            .await
            .ok_or(AuthenticationError::ShuttingDown)?
            .map_err(AuthenticationError::AuthorityProxy)?;

        let subject = Subject::new_for_owner(
            cred.pid()
                .try_into()
                .map_err(AuthenticationError::NumberConversion)?,
            None,
            Some(cred.uid()),
        )
        .map_err(AuthenticationError::Subject)?;

        let timeout = tokio::time::sleep(USER_INTERACTION_TIMEOUT);
        wait_for_authorization(
            AuthProxy { proxy, subject },
            self.shutdown_token.clone(),
            timeout,
        )
        .await
    }
}

async fn wait_for_authorization(
    proxy: impl AuthorizationChecker,
    shutdown_token: CancellationToken,
    timeout: impl Future<Output = ()>,
) -> Result<AuthorizationResult, AuthenticationError> {
    let check_authorization_fut = proxy.check_authorization();

    tokio::select! {
        biased;
        auth_result = check_authorization_fut => {
            auth_result
        }
        _ = timeout => {
            tracing::warn!("User authorization timed out");
            proxy.cancel_check_authorization().await?;
            Err(AuthenticationError::Timeout)
        }
        _ = shutdown_token.cancelled() => {
            tracing::debug!("Received shutdown signal");
            // We do a best effort to cancel the authorization before shutting down
            proxy.cancel_check_authorization().await.ok();
            Err(AuthenticationError::ShuttingDown)
        }
    }
}

// Check that the user can authenticate via system password
// This function depends on user interaction, so it must ensure it doesn't await
// indefinitely and starve the consumer.
pub(crate) async fn is_authenticated(
    stream: &mut Transport,
    auth_material: AuthenticationMaterial,
) -> Result<(), AuthenticationError> {
    let cred = getsockopt(stream, PeerCredentials).map_err(AuthenticationError::GetSockOpt)?;
    if user_in_group(cred.uid().into(), auth_material.nym_vpn_gid) {
        tracing::debug!("User is part of the nym-vpn group");
        Ok(())
    } else {
        authenticate_with_prompt(cred, PolkitPrompter::new(auth_material.shutdown_token)).await
    }
}

fn user_in_group(uid: Uid, gid: Option<Gid>) -> bool {
    if uid.is_root() {
        tracing::trace!("User is root");
        return true;
    }

    let Ok(Some(user)) = User::from_uid(uid) else {
        tracing::debug!("User {uid} could not be parsed or it disappeared");
        return false;
    };
    let Some(gid) = gid else {
        tracing::debug!("No nym-vpn group");
        return false;
    };
    if user.gid == gid {
        tracing::trace!("User is primary of the group");
        return true;
    }

    let Ok(name) = CString::new(user.name.as_bytes()) else {
        tracing::warn!("User name could not be parsed into CString");
        return false;
    };
    let Ok(group_list) = getgrouplist(&name, user.gid)
        .inspect_err(|err| tracing::warn!("Could not get the group list: {err:?}"))
    else {
        return false;
    };

    let in_group = group_list.contains(&gid);
    if !in_group {
        tracing::info!(
            "Connecting user is not in the nym-vpn UNIX group. If they would be added, prompt authentication would not be needed anymore"
        );
    }

    in_group
}

async fn authenticate_with_prompt(
    cred: UnixCredentials,
    prompter: impl Prompter,
) -> Result<(), AuthenticationError> {
    let auth_result = prompter.prompt_for_authorization(cred).await?;

    if auth_result.is_authorized {
        Ok(())
    } else {
        Err(AuthenticationError::AuthorizationDenied)
    }
}

fn skip_authentication_checks(disable_client_verification: bool) -> bool {
    cfg!(debug_assertions) || disable_client_verification
}

pub(crate) fn incoming(
    uds: Uds,
    auth_material: AuthenticationMaterial,
) -> impl Stream<Item = std::io::Result<Transport>> {
    let shutdown_token = auth_material.shutdown_token.clone();
    let auth_material = if skip_authentication_checks(auth_material.disable_client_verification) {
        None
    } else {
        Some(auth_material)
    };
    let auth_layer = AuthenticationLayer::new(uds, auth_material, shutdown_token);
    auth_layer.stream()
}

#[cfg(test)]
mod tests {
    use std::{
        future::pending,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        task::Poll,
    };

    use tokio::{
        net::UnixStream,
        sync::{Mutex, RwLock},
    };

    use crate::{auth_result::AuthenticaticationResult, authentication::authorize};

    use super::*;

    struct MockProxy {
        user_authorization: Arc<RwLock<bool>>,
        check_tried: bool,
        cancelled: bool,
    }

    impl MockProxy {
        fn new(user_authorization: Arc<RwLock<bool>>) -> Arc<Mutex<Self>> {
            Arc::new(Mutex::new(Self {
                user_authorization,
                check_tried: false,
                cancelled: false,
            }))
        }
    }

    #[async_trait::async_trait]
    impl AuthorizationChecker for &Arc<Mutex<MockProxy>> {
        async fn check_authorization(&self) -> Result<AuthorizationResult, AuthenticationError> {
            let mut inner = self.lock().await;
            inner.check_tried = true;
            inner.cancelled = false;
            Ok(AuthorizationResult {
                is_authorized: *inner.user_authorization.read().await,
                is_challenge: Default::default(),
                details: Default::default(),
            })
        }

        async fn cancel_check_authorization(&self) -> Result<(), AuthenticationError> {
            self.lock().await.cancelled = true;
            Ok(())
        }
    }

    #[derive(Clone)]
    struct MockSleeper {
        ready: Arc<AtomicBool>,
    }

    impl MockSleeper {
        fn new() -> Self {
            Self {
                ready: Arc::new(AtomicBool::new(false)),
            }
        }

        // activates the timeout
        fn timeout(&self) {
            self.ready.fetch_or(true, Ordering::SeqCst);
        }
    }

    impl Future for MockSleeper {
        type Output = ();

        fn poll(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> Poll<Self::Output> {
            if self.ready.load(Ordering::SeqCst) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }

    struct MockPrompter {
        is_authorized: bool,
    }

    #[async_trait::async_trait]
    impl Prompter for MockPrompter {
        async fn prompt_for_authorization(
            &self,
            _cred: UnixCredentials,
        ) -> Result<AuthorizationResult, AuthenticationError> {
            Ok(AuthorizationResult {
                is_authorized: self.is_authorized,
                is_challenge: Default::default(),
                details: Default::default(),
            })
        }
    }

    #[tokio::test]
    async fn authorized() {
        let (mut client, mut server) = UnixStream::pair().unwrap();
        authorize(&mut server).await;
        let ret = AuthenticaticationResult::recv(&mut client).await;
        assert!(matches!(ret, AuthenticaticationResult::Accepted));
    }

    #[tokio::test]
    async fn wait_for_authorized() {
        let proxy = MockProxy::new(Arc::new(RwLock::new(true)));
        let res = wait_for_authorization(&proxy, CancellationToken::new(), pending())
            .await
            .unwrap();
        assert!(proxy.lock().await.check_tried);
        assert!(res.is_authorized);
    }

    #[tokio::test]
    async fn wait_for_denied() {
        let proxy = MockProxy::new(Arc::new(RwLock::new(false)));
        let res = wait_for_authorization(&proxy, CancellationToken::new(), pending())
            .await
            .unwrap();
        assert!(proxy.lock().await.check_tried);
        assert!(!res.is_authorized);
    }

    #[tokio::test]
    async fn cancel_wait() {
        let user_waiting = Arc::new(RwLock::new(false));
        let proxy = MockProxy::new(user_waiting.clone());
        let cancellation_token = CancellationToken::new();

        // Make it wait for user input indefinitely
        let _user_input = user_waiting.write().await;
        let cloned_proxy = proxy.clone();
        let cloned_token = cancellation_token.clone();
        let handle: tokio::task::JoinHandle<Result<AuthorizationResult, AuthenticationError>> =
            tokio::spawn(async move {
                wait_for_authorization(&cloned_proxy, cloned_token, pending()).await
            });
        cancellation_token.cancel();
        let res = handle.await.unwrap();

        assert!(proxy.lock().await.cancelled);
        assert!(matches!(res, Err(AuthenticationError::ShuttingDown)));
    }

    #[tokio::test]
    async fn timed_out_wait() {
        let user_waiting = Arc::new(RwLock::new(false));
        let proxy = MockProxy::new(user_waiting.clone());
        let timeout = MockSleeper::new();

        // Make it wait for user input indefinitely
        let _user_input = user_waiting.write().await;
        let cloned_proxy = proxy.clone();
        let cloned_timeout = timeout.clone();
        let handle = tokio::spawn(async move {
            wait_for_authorization(&cloned_proxy, CancellationToken::new(), cloned_timeout).await
        });
        timeout.timeout();
        let res = handle.await.unwrap();

        assert!(matches!(res, Err(AuthenticationError::Timeout)));
    }

    #[tokio::test]
    async fn authorized_by_prompt() {
        let (_, server) = UnixStream::pair().unwrap();
        authenticate_with_prompt(
            getsockopt(&server, PeerCredentials).unwrap(),
            MockPrompter {
                is_authorized: true,
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn denied_by_prompt() {
        let (_, server) = UnixStream::pair().unwrap();
        let err = authenticate_with_prompt(
            getsockopt(&server, PeerCredentials).unwrap(),
            MockPrompter {
                is_authorized: false,
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AuthenticationError::AuthorizationDenied));
    }

    #[test]
    // Test builds are debug and are authorized
    fn unsigned_build_authorized() {
        assert!(skip_authentication_checks(false));
    }

    #[test]
    fn embedded_policy_matches_action_id() {
        assert!(
            POLKIT_POLICY.contains(ACTION_ID),
            "the bundled policy XML must declare the action id used at runtime",
        );
    }

    #[tokio::test]
    async fn install_polkit_policy_writes_first_writable_dir() {
        let primary = tempfile::tempdir().unwrap();
        let fallback = tempfile::tempdir().unwrap();
        let primary_path = primary.path().to_str().unwrap().to_owned();
        let fallback_path = fallback.path().to_str().unwrap().to_owned();
        let fallback_dirs: &[&str] = &[fallback_path.as_str()];

        let written = install_polkit_policy(primary_path.as_str(), fallback_dirs)
            .await
            .unwrap();

        let expected = format!("{primary_path}/{ACTION_ID}.policy");
        assert_eq!(written, expected);
        assert!(std::fs::metadata(&expected).is_ok());
        // Fallback directory must remain untouched when primary succeeds.
        let fallback_file = format!("{fallback_path}/{ACTION_ID}.policy");
        assert!(std::fs::metadata(&fallback_file).is_err());
    }

    #[tokio::test]
    async fn install_polkit_policy_falls_back_when_primary_unwritable() {
        let primary = "/nonexistent/nym-vpn-test-readonly-primary";
        let fallback = tempfile::tempdir().unwrap();
        let fallback_path = fallback.path().to_str().unwrap().to_owned();
        let fallback_dirs: &[&str] = &[fallback_path.as_str()];

        let written = install_polkit_policy(primary, fallback_dirs).await.unwrap();

        let expected = format!("{fallback_path}/{ACTION_ID}.policy");
        assert_eq!(written, expected);
        let content = std::fs::read_to_string(&expected).unwrap();
        assert!(content.contains(ACTION_ID));
    }

    #[tokio::test]
    async fn install_polkit_policy_creates_missing_parent_dir() {
        let parent = tempfile::tempdir().unwrap();
        // Point at a yet-to-be-created subdirectory under the tempdir so we
        // exercise the create_dir_all branch without hitting privileged paths.
        let dir = parent
            .path()
            .join("polkit-1/actions")
            .to_str()
            .unwrap()
            .to_owned();

        let written = install_polkit_policy(dir.as_str(), &[]).await.unwrap();

        let expected = format!("{dir}/{ACTION_ID}.policy");
        assert_eq!(written, expected);
        assert!(std::fs::metadata(&expected).is_ok());
    }

    #[tokio::test]
    async fn install_polkit_policy_errors_when_no_dir_writable() {
        let dirs: &[&str] = &[
            "/nonexistent/nym-vpn-test-readonly-a",
            "/nonexistent/nym-vpn-test-readonly-b",
        ];

        let err = install_polkit_policy(dirs[0], &[dirs[1]])
            .await
            .unwrap_err();
        assert!(matches!(err, AuthenticationError::CreateActionPolicy(_)));
    }
}
