// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use nix::sys::socket::{getsockopt, sockopt::PeerCredentials};
use nym_ipc_client::authentication::AuthenticaticationResult;
use tokio::net::UnixStream;
use tokio_util::sync::CancellationToken;
use zbus::Connection;
use zbus_polkit::policykit1::{
    AuthorityProxy, AuthorizationResult, CheckAuthorizationFlags, Subject,
};

use crate::authentication::error::AuthenticationError;

const ACTION_ID: &str = "com.nymvpn.vpnd.unix-access";
const CANCELLATION_ID: &str = "com.nymvpn.vpnd.cancel";
const USER_INTERACTION_TIMEOUT: Duration = Duration::from_secs(60);

async fn wait_for_authorization(
    proxy: AuthorityProxy<'_>,
    subject: Subject,
    shutdown_token: CancellationToken,
    // stream_shutdown_token: CancellationToken,
) -> Result<AuthorizationResult, AuthenticationError> {
    // details might be useful to set some locale-sensitive messages and icon images in the authentication dialog
    let details = std::collections::HashMap::new();
    let timer = tokio::time::sleep(USER_INTERACTION_TIMEOUT);
    let check_authorization_fut = proxy.check_authorization(
        &subject,
        ACTION_ID,
        &details,
        CheckAuthorizationFlags::AllowUserInteraction.into(),
        CANCELLATION_ID,
    );

    tokio::select! {
        biased;
        auth_result = check_authorization_fut => {
            auth_result.map_err(AuthenticationError::CheckAuthorization)
        }
        _ = timer => {
            tracing::warn!("No user authorization for {:?}", USER_INTERACTION_TIMEOUT);
            proxy.cancel_check_authorization(CANCELLATION_ID).await.map_err(AuthenticationError::CancelAuthorization)?;
            Err(AuthenticationError::Timeout)
        }
        _ = shutdown_token.cancelled() => {
            tracing::debug!("Received shutdown signal");
            // We do a best effort to cancel the authorization before shutting down
            proxy.cancel_check_authorization(CANCELLATION_ID).await.ok();
            Err(AuthenticationError::ShuttingDown)
        }
    }
}

// Return back the stream if the authentication succeeded, and `None` otherwise
// This function depends on user interaction, so it must ensure it doesn't await
// indefinitely and starve the consumer.
pub(crate) async fn is_authenticated(
    mut stream: UnixStream,
    shutdown_token: CancellationToken,
) -> Result<UnixStream, AuthenticationError> {
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

    let cred = getsockopt(&stream, PeerCredentials).map_err(AuthenticationError::GetSockOpt)?;
    let subject = Subject::new_for_owner(
        cred.pid()
            .try_into()
            .map_err(AuthenticationError::NumberConversion)?,
        None,
        Some(cred.uid()),
    )
    .map_err(AuthenticationError::Subject)?;

    let auth_result = wait_for_authorization(proxy, subject, shutdown_token).await?;

    if auth_result.is_authorized {
        AuthenticaticationResult::Accepted.send(&mut stream).await;
        Ok(stream)
    } else {
        AuthenticaticationResult::Denied.send(&mut stream).await;
        Err(AuthenticationError::AuthorizationDenied)
    }
}
