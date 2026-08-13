// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::{Transport, incoming, is_authenticated};
#[cfg(target_os = "linux")]
use nix::unistd::{Gid, Group};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::SigningRequirements;
#[allow(unused)]
#[cfg(target_os = "macos")]
pub(crate) use macos::{Transport, incoming, is_authenticated, skip_authentication_checks};

#[cfg(target_os = "windows")]
mod windows;
use tokio::io::AsyncWrite;
#[cfg(target_os = "windows")]
pub(crate) use windows::{Transport, incoming, is_authenticated};

pub(crate) mod error;

use async_stream::try_stream;
use tokio_stream::{Stream, StreamExt};

use std::io::Result;

use crate::auth_result::{AuthenticaticationQuery, AuthenticaticationResult};

pub(crate) async fn authorize(stream: impl AsyncWrite + Unpin) {
    AuthenticaticationResult::Accepted.send(stream).await;
}

pub(crate) async fn deny(stream: impl AsyncWrite + Unpin) {
    AuthenticaticationResult::Denied.send(stream).await;
}

#[derive(Clone)]
pub struct AuthenticationLayer<T> {
    listener: T,
    auth_material: Option<AuthenticationMaterial>,
    #[cfg(target_os = "linux")]
    shutdown_token: tokio_util::sync::CancellationToken,
}

impl<T> AuthenticationLayer<T> {
    fn new(
        listener: T,
        auth_material: Option<AuthenticationMaterial>,
        #[cfg(target_os = "linux")] shutdown_token: tokio_util::sync::CancellationToken,
    ) -> Self {
        Self {
            listener,
            auth_material,
            #[cfg(target_os = "linux")]
            shutdown_token,
        }
    }
}

#[derive(Clone)]
#[allow(unused)]
pub struct AuthenticationMaterial {
    pub(crate) disable_client_verification: bool,
    #[cfg(target_os = "windows")]
    pub(crate) nym_certificate_serial_number: String,
    #[cfg(target_os = "macos")]
    pub(crate) signing_requirements: SigningRequirements,
    #[cfg(target_os = "linux")]
    pub(crate) nym_vpn_gid: Option<Gid>,
    #[cfg(unix)]
    pub(crate) shutdown_token: tokio_util::sync::CancellationToken,
}

impl AuthenticationMaterial {
    pub fn new(
        disable_client_verification: bool,
        #[cfg(target_os = "windows")] nym_certificate_serial_number: String,
        #[cfg(target_os = "macos")] signing_requirements: SigningRequirements,
        #[cfg(target_os = "linux")] nym_vpn_group: &str,
        #[cfg(unix)] shutdown_token: tokio_util::sync::CancellationToken,
    ) -> Self {
        #[cfg(target_os = "linux")]
        let nym_vpn_gid = Group::from_name(nym_vpn_group)
            .inspect_err(|err| tracing::warn!("Could not get the group {nym_vpn_group}: {err}"))
            .inspect(|g| {
                if g.is_none() {
                    tracing::warn!("Not group found for {nym_vpn_group}")
                }
            })
            .ok()
            .flatten()
            .map(|g| g.gid);

        Self {
            disable_client_verification,
            #[cfg(target_os = "windows")]
            nym_certificate_serial_number,
            #[cfg(target_os = "macos")]
            signing_requirements,
            #[cfg(target_os = "linux")]
            nym_vpn_gid,
            #[cfg(unix)]
            shutdown_token,
        }
    }
}

async fn authorized_stream(
    stream: &mut Transport,
    auth_material: Option<AuthenticationMaterial>,
) -> bool {
    if !AuthenticaticationQuery::recv(&mut *stream).await.status() {
        tracing::warn!("Query not recognized");
    }
    let Some(auth_material) = auth_material else {
        tracing::debug!("Skipping authentication checks");
        authorize(stream).await;
        return true;
    };
    match is_authenticated(stream, auth_material).await {
        Ok(()) => {
            authorize(stream).await;
            tracing::debug!("Client connection got authorized");
            true
        }
        Err(err) => {
            deny(stream).await;
            // Surface auth failures at warn so they appear in the default
            // log level. Without this the actual reason for a stuck
            // "Authentication required" modal is hidden behind RUST_LOG=debug.
            tracing::warn!("Connection did not get authenticated: {err:?}");
            false
        }
    }
}

impl<T: Unpin + Stream<Item = Result<Transport>>> AuthenticationLayer<T> {
    fn stream(mut self) -> impl Stream<Item = Result<Transport>> {
        try_stream! {
            loop {
                #[cfg(not(target_os = "linux"))]
                let shutdown_signal = std::future::pending::<()>();
                #[cfg(target_os = "linux")]
                let shutdown_signal = self.shutdown_token.cancelled();

                let next_stream = tokio::select! {
                    _ = shutdown_signal => {
                        break;
                    }
                    stream = self.listener.next() => {
                        stream
                    }
                };
                let Some(stream) = next_stream else {
                    break;
                };
                let mut stream = stream?;
                if authorized_stream(&mut stream, self.auth_material.clone()).await {
                    yield stream;
                }

            }
        }
    }
}
