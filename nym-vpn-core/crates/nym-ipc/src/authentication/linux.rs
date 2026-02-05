// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use nix::sys::socket::{UnixCredentials, getsockopt, sockopt::PeerCredentials};
use tokio::net::UnixStream;
use tokio_util::sync::CancellationToken;
use zbus::Connection;
use zbus_polkit::policykit1::{
    AuthorityProxy, AuthorizationResult, CheckAuthorizationFlags, Subject,
};

use crate::{auth_result::AuthenticaticationResult, authentication::error::AuthenticationError};

const ACTION_ID: &str = "com.nymvpn.vpnd.unix-access";
const CANCELLATION_ID: &str = "com.nymvpn.vpnd.cancel";
const USER_INTERACTION_TIMEOUT: Duration = Duration::from_secs(60);

#[async_trait::async_trait]
trait AuthorizationChecker {
    async fn check_authorization(&self) -> Result<AuthorizationResult, zbus::Error>;
    async fn cancel_check_authorization(&self) -> Result<(), zbus::Error>;
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
    async fn check_authorization(&self) -> Result<AuthorizationResult, zbus::Error> {
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
    }
    async fn cancel_check_authorization(&self) -> Result<(), zbus::Error> {
        self.proxy.cancel_check_authorization(CANCELLATION_ID).await
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
            auth_result.map_err(AuthenticationError::CheckAuthorization)
        }
        _ = timeout => {
            tracing::warn!("User authorization timed out");
            proxy.cancel_check_authorization().await.map_err(AuthenticationError::CancelAuthorization)?;
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

async fn authorize(stream: &mut UnixStream) {
    AuthenticaticationResult::Accepted.send(stream).await;
}

// Return back the stream if the authentication succeeded, and `None` otherwise
// This function depends on user interaction, so it must ensure it doesn't await
// indefinitely and starve the consumer.
pub(crate) async fn is_authenticated(
    mut stream: UnixStream,
    shutdown_token: CancellationToken,
) -> Result<UnixStream, AuthenticationError> {
    // Let debug builds skip authorization process
    // TODO: Disable feature gating once front-end prevents spamming
    if cfg!(debug_assertions) || cfg!(not(feature = "authentication")) {
        authorize(&mut stream).await;
        return Ok(stream);
    }
    authenticate_with_prompt(stream, PolkitPrompter::new(shutdown_token)).await
}

async fn authenticate_with_prompt(
    mut stream: UnixStream,
    prompter: impl Prompter,
) -> Result<UnixStream, AuthenticationError> {
    let cred = getsockopt(&stream, PeerCredentials).map_err(AuthenticationError::GetSockOpt)?;
    let auth_result = prompter.prompt_for_authorization(cred).await?;

    if auth_result.is_authorized {
        authorize(&mut stream).await;
        Ok(stream)
    } else {
        AuthenticaticationResult::Denied.send(&mut stream).await;
        Err(AuthenticationError::AuthorizationDenied)
    }
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
        async fn check_authorization(&self) -> Result<AuthorizationResult, zbus::Error> {
            let mut inner = self.lock().await;
            inner.check_tried = true;
            inner.cancelled = false;
            Ok(AuthorizationResult {
                is_authorized: *inner.user_authorization.read().await,
                is_challenge: Default::default(),
                details: Default::default(),
            })
        }

        async fn cancel_check_authorization(&self) -> Result<(), zbus::Error> {
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
    // Debug builds (like tests or dev runs) are automatically authorized
    async fn debug_build_authorized() {
        let (mut client, server) = UnixStream::pair().unwrap();
        is_authenticated(server, CancellationToken::new())
            .await
            .unwrap();
        let client_res = AuthenticaticationResult::recv(&mut client).await;
        assert!(client_res.accepted());
    }

    #[tokio::test]
    async fn authorized_by_prompt() {
        let (mut client, server) = UnixStream::pair().unwrap();
        authenticate_with_prompt(
            server,
            MockPrompter {
                is_authorized: true,
            },
        )
        .await
        .unwrap();
        let client_res = AuthenticaticationResult::recv(&mut client).await;
        assert!(client_res.accepted());
    }

    #[tokio::test]
    async fn denied_by_prompt() {
        let (mut client, server) = UnixStream::pair().unwrap();
        let err = authenticate_with_prompt(
            server,
            MockPrompter {
                is_authorized: false,
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AuthenticationError::AuthorizationDenied));

        let client_res = AuthenticaticationResult::recv(&mut client).await;
        assert!(!client_res.accepted());
    }
}
