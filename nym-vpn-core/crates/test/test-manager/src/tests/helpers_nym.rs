// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{Error, WAIT_FOR_TUNNEL_STATE_TIMEOUT, config_nym::TEST_CONFIG_NYM};
use anyhow::Context;
use futures::StreamExt;
use nym_vpn_lib_types::{TunnelEvent, TunnelState};
use nym_vpn_proto::rpc_client::{Error as NymClientError, RpcClient as NymProxyClient};
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub enum ExpectedTunnelState {
    Connected,
    Disconnected,
    Connecting,
    Disconnecting,
    Offline,
    Error(String),
}

impl From<TunnelState> for ExpectedTunnelState {
    fn from(value: TunnelState) -> Self {
        match value {
            TunnelState::Connected { .. } => ExpectedTunnelState::Connected,
            TunnelState::Disconnected => ExpectedTunnelState::Disconnected,
            TunnelState::Connecting { .. } => ExpectedTunnelState::Connecting,
            TunnelState::Disconnecting { .. } => ExpectedTunnelState::Disconnecting,
            TunnelState::Offline { .. } => ExpectedTunnelState::Offline,
            TunnelState::Error(reason) => ExpectedTunnelState::Error(reason.to_string()),
        }
    }
}

pub const THROTTLE_RETRY_DELAY: Duration = Duration::from_secs(120);

/// Log in and retry if it fails due to throttling
pub async fn login_with_retries(nym_client: &mut NymProxyClient) -> Result<(), NymClientError> {
    log::debug!("Logging in/generating device");
    // TODO dz loop is to avoid throttling (inherited from mullvad)
    // is this necessary?
    loop {
        match nym_client
            .store_account_friendly(&TEST_CONFIG_NYM.mnemonic)
            .await
        {
            Err(NymClientError::Rpc(status))
                if status.message().to_uppercase().contains("THROTTLED") =>
            {
                // Work around throttling errors by sleeping
                log::debug!(
                    "Login failed due to throttling. Sleeping for {} seconds",
                    THROTTLE_RETRY_DELAY.as_secs()
                );

                tokio::time::sleep(THROTTLE_RETRY_DELAY).await;
            }
            Err(err) => return Err(err),
            Ok(_) => break,
        }
    }

    // nym_client.reset_device_identity(None).await?;

    Ok(())
}

pub async fn ensure_logged_in(nym_client: &mut NymProxyClient) -> anyhow::Result<()> {
    log::info!("Ensuring we're logged in by logging out and back in...");

    nym_client
        .forget_account()
        .await
        .context("Failed to forget account")?;

    // re-log in...
    login_with_retries(nym_client)
        .await
        .context("Failed to log in")?;

    let active_devices = nym_client.get_active_devices().await?;
    if nym_client.is_account_stored().await? && !active_devices.is_empty() {
        return Ok(());
    }

    Ok(())
}

pub async fn disconnect_and_wait(nym_client: &mut NymProxyClient) -> Result<(), Error> {
    log::trace!("Disconnecting");
    nym_client.disconnect_tunnel().await?;

    wait_for_tunnel_state_fn(
        nym_client,
        |state| matches!(state, TunnelState::Disconnected),
        WAIT_FOR_TUNNEL_STATE_TIMEOUT,
    )
    .await?;

    log::trace!("Disconnected");

    Ok(())
}

/// Wait for the tunnel to reach a state accepted by `accept_state_fn`, using the daemon event
/// stream. We subscribe to events before checking the current state, so no transitions are missed.
pub async fn wait_for_tunnel_state_fn(
    rpc: &mut NymProxyClient,
    accept_state_fn: impl Fn(&TunnelState) -> bool,
    timeout: Duration,
) -> Result<TunnelState, Error> {
    let mut events = rpc
        .listen_to_events()
        .await
        .map_err(|status| Error::Daemon(format!("Failed to get event stream: {status}")))?;

    let state = rpc
        .get_tunnel_state()
        .await
        .map_err(|error| Error::Daemon(format!("Failed to get tunnel state: {error:?}")))?;

    log::debug!("Current tunnel state: {state:?}");

    if accept_state_fn(&state) {
        return Ok(state);
    }

    tokio::time::timeout(timeout, async {
        loop {
            match events.next().await {
                Some(Ok(TunnelEvent::NewState(state))) if accept_state_fn(&state) => {
                    log::debug!("Reached expected tunnel state: {state:?}");
                    break Ok(state);
                }
                Some(Ok(event)) => {
                    log::debug!("Ignoring tunnel event: {event:?}");
                    continue;
                }
                Some(Err(status)) => {
                    break Err(Error::Daemon(format!("Failed to get next event: {status}")));
                }
                None => break Err(Error::Daemon(String::from("Lost daemon event stream"))),
            }
        }
    })
    .await
    .map_err(|_| Error::Daemon(String::from("Tunnel event listener timed out")))?
}

pub async fn wait_for_tunnel_state(
    rpc: &mut NymProxyClient,
    expected: ExpectedTunnelState,
) -> Result<TunnelState, Error> {
    wait_for_tunnel_state_with_timeout(rpc, expected, Duration::from_secs(60)).await
}

async fn wait_for_tunnel_state_with_timeout(
    rpc: &mut NymProxyClient,
    expected: ExpectedTunnelState,
    timeout: Duration,
) -> Result<TunnelState, Error> {
    log::debug!(
        "Waiting for tunnel state: {expected:?} (timeout: {}s)",
        timeout.as_secs()
    );
    wait_for_tunnel_state_fn(
        rpc,
        move |state| ExpectedTunnelState::from(state.clone()) == expected,
        timeout,
    )
    .await
}
