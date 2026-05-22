// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use nix::sys::socket::{UnixCredentials, getsockopt, sockopt::PeerCredentials};
use tokio::{fs::File, io::AsyncWriteExt, net::UnixStream};
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
const POLKIT_POLICY: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<policyconfig>
  <action id="com.nymvpn.vpnd.unix-access">
    <description>Connect via unix socket</description>
    <message>Authentication is required to connect to the daemon</message>

    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_self</allow_active>
    </defaults>
  </action>
</policyconfig>
"#;

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
    pub action_id: String,
    /// Policy XML to install if the action is not yet registered with polkit.
    pub policy_xml: String,
}

#[async_trait::async_trait]
impl AuthorizationChecker for AuthProxy<'_> {
    async fn check_authorization(&self) -> Result<AuthorizationResult, AuthenticationError> {
        if !self
            .proxy
            .enumerate_actions("")
            .await
            .map_err(AuthenticationError::EnumerateActions)?
            .iter()
            .any(|action| action.action_id == self.action_id)
        {
            let mut file =
                File::create(format!("/usr/share/polkit-1/actions/{}.policy", self.action_id))
                    .await
                    .map_err(AuthenticationError::CreateActionPolicy)?;
            file.write_all(self.policy_xml.as_bytes())
                .await
                .map_err(AuthenticationError::WriteActionPolicy)?;
        }

        // details might be useful to set some locale-sensitive messages and icon images in the authentication dialog
        let details = std::collections::HashMap::new();
        self.proxy
            .check_authorization(
                &self.subject,
                &self.action_id,
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
            AuthProxy {
                proxy,
                subject,
                action_id: ACTION_ID.to_string(),
                policy_xml: POLKIT_POLICY.to_string(),
            },
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

/// Return [`UnixCredentials`] for the current process (pid, uid, gid).
///
/// Used as a fallback subject for polkit when the calling client's peer
/// credentials are not available (e.g. from a tonic gRPC handler that does not
/// yet thread unix socket peer credentials through the request extensions).
pub fn self_credentials() -> UnixCredentials {
    UnixCredentials::new()
}

/// Request per-call polkit authorization for an arbitrary action id.
/// Installs the policy file on first use if it doesn't already exist.
/// Returns Ok if the user authenticated; Err otherwise.
///
/// NOTE: the `pid` and `uid` in `cred` are used as the Subject for polkit.
/// Currently callers that don't have access to the peer unix-socket credentials
/// (e.g. tonic gRPC handlers) pass the daemon's own process credentials
/// (`std::process::id()`, `nix::unistd::getuid()`).  The polkit prompt still
/// fires and authenticates whoever is at the keyboard; the only difference is
/// that the *subject* reported to polkit is the daemon rather than the actual
/// client.
///
/// TODO: thread real peer credentials through the tonic request extensions so
/// that this function receives the calling client's pid/uid instead.
pub async fn request_action_authorization(
    cred: UnixCredentials,
    action_id: &str,
    policy_xml: &str,
    shutdown_token: CancellationToken,
) -> Result<(), AuthenticationError> {
    let connection = shutdown_token
        .run_until_cancelled(Connection::system())
        .await
        .ok_or(AuthenticationError::ShuttingDown)?
        .map_err(AuthenticationError::MessageBusConnection)?;
    let proxy = shutdown_token
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

    let auth_proxy = AuthProxy {
        proxy,
        subject,
        action_id: action_id.to_string(),
        policy_xml: policy_xml.to_string(),
    };
    let timeout = tokio::time::sleep(USER_INTERACTION_TIMEOUT);
    let auth_result = wait_for_authorization(auth_proxy, shutdown_token, timeout).await?;

    if auth_result.is_authorized {
        Ok(())
    } else {
        Err(AuthenticationError::AuthorizationDenied)
    }
}

// Check that the user can authenticate via system password
// This function depends on user interaction, so it must ensure it doesn't await
// indefinitely and starve the consumer.
pub(crate) async fn is_authenticated(
    stream: &mut Transport,
    auth_material: AuthenticationMaterial,
) -> Result<(), AuthenticationError> {
    authenticate_with_prompt(stream, PolkitPrompter::new(auth_material.shutdown_token)).await
}

async fn authenticate_with_prompt(
    stream: &mut UnixStream,
    prompter: impl Prompter,
) -> Result<(), AuthenticationError> {
    let cred = getsockopt(stream, PeerCredentials).map_err(AuthenticationError::GetSockOpt)?;
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

    use tokio::sync::{Mutex, RwLock};

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
        let (_, mut server) = UnixStream::pair().unwrap();
        authenticate_with_prompt(
            &mut server,
            MockPrompter {
                is_authorized: true,
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn denied_by_prompt() {
        let (_, mut server) = UnixStream::pair().unwrap();
        let err = authenticate_with_prompt(
            &mut server,
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

    // `request_action_authorization` constructs its own `AuthorityProxy` over
    // a real D-Bus connection, so it cannot be exercised by the MockProxy
    // scaffolding above.  The function's logic (timeout, shutdown, denied /
    // authorized branching) is fully covered by the `wait_for_authorization`
    // tests above because `request_action_authorization` delegates to that
    // function.
    //
    // TODO(phase-12): add an integration test that calls
    // `request_action_authorization` end-to-end via a polkit test double or
    // a real system D-Bus session when running in CI.
}
