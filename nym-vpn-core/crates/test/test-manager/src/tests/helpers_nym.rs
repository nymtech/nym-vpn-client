// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{
    Error, WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT, WAIT_FOR_TUNNEL_STATE_TIMEOUT,
    config_nym::TEST_CONFIG_NYM,
};
use futures::StreamExt;
use nym_vpn_lib_types::{AccountCommandError, AccountControllerState, TunnelEvent, TunnelState};
use nym_vpn_proto::rpc_client::{Error as NymClientError, RpcClient as NymProxyClient};
use std::{net::SocketAddr, time::Duration};
use test_rpc::NymServiceClient;

/// Bounded best-effort disconnect after a tunnel wait timeout. Must not nest a full
/// `disconnect_and_wait` (that would block the suite for another 40s on a dead serial).
const BEST_EFFORT_DISCONNECT_TIMEOUT: Duration = Duration::from_secs(10);

const TUNNEL_STATE_POLL_ATTEMPTS: u32 = 3;
const TUNNEL_STATE_POLL_DELAY: Duration = Duration::from_millis(500);

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

pub async fn login_idempotent(nym_client: &mut NymProxyClient) -> anyhow::Result<()> {
    match nym_client
        .get_account_state()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get account state: {e}"))?
    {
        AccountControllerState::ReadyToConnect => {
            return Ok(());
        }
        AccountControllerState::LoggedOut => {
            store_account_idempotent(nym_client).await?;
        }
        _ => {
            // for other states just wait, AC will either reach ReadyToConnect or we'll timeout.
        }
    }
    wait_for_account_state(nym_client, AccountControllerState::ReadyToConnect)
        .await
        .map(drop)
        .map_err(From::from)
}

async fn store_account_idempotent(nym_client: &mut NymProxyClient) -> anyhow::Result<()> {
    loop {
        let request = nym_vpn_lib_types::StoreAccountRequest::Vpn {
            mnemonic: TEST_CONFIG_NYM.mnemonic.to_string(),
        };
        match nym_client.store_account(request).await {
            Ok(response) => match response.error {
                None | Some(AccountCommandError::ExistingAccount) => break,
                Some(err) => anyhow::bail!("store_account error: {err}"),
            },
            Err(NymClientError::Rpc(status))
                if status.message().to_uppercase().contains("THROTTLED") =>
            {
                log::debug!(
                    "Login failed due to throttling. Sleeping for {} seconds",
                    THROTTLE_RETRY_DELAY.as_secs()
                );
                tokio::time::sleep(THROTTLE_RETRY_DELAY).await;
            }
            Err(err) => anyhow::bail!("store_account RPC failed: {err}"),
        }
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
        false,
    )
    .await?;

    log::trace!("Disconnected");

    Ok(())
}

/// Best-effort `disconnect_tunnel` with a short deadline. Never waits for Disconnected.
pub async fn best_effort_disconnect(nym_client: &mut NymProxyClient) {
    match tokio::time::timeout(
        BEST_EFFORT_DISCONNECT_TIMEOUT,
        nym_client.disconnect_tunnel(),
    )
    .await
    {
        Ok(Ok(_)) => log::info!("Best-effort disconnect_tunnel succeeded"),
        Ok(Err(err)) => log::warn!("Best-effort disconnect_tunnel failed: {err}"),
        Err(_) => log::warn!(
            "Best-effort disconnect_tunnel timed out after {}s",
            BEST_EFFORT_DISCONNECT_TIMEOUT.as_secs()
        ),
    }
}

pub async fn wait_for_tunnel_state(
    rpc: &mut NymProxyClient,
    expected: ExpectedTunnelState,
) -> Result<TunnelState, Error> {
    let (timeout, disconnect_on_timeout) = tunnel_wait_params(&expected);
    log::debug!(
        "Waiting for tunnel state: {expected:?} (timeout: {}s)",
        timeout.as_secs()
    );
    wait_for_tunnel_state_fn(
        rpc,
        move |state| ExpectedTunnelState::from(state.clone()) == expected,
        timeout,
        disconnect_on_timeout,
    )
    .await
}

/// Connect waits get a longer timeout and disconnect-on-timeout; other waits do not.
pub(crate) fn tunnel_wait_params(expected: &ExpectedTunnelState) -> (Duration, bool) {
    match expected {
        ExpectedTunnelState::Connected | ExpectedTunnelState::Connecting => {
            (WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT, true)
        }
        _ => (WAIT_FOR_TUNNEL_STATE_TIMEOUT, false),
    }
}

/// Wait for the tunnel to reach a state accepted by `accept_state_fn`, using the daemon event
/// stream. We subscribe to events before checking the current state, so no transitions are missed.
///
/// On timeout, the event stream is dropped before follow-up unary RPCs so the serial mux can
/// accept `get_tunnel_state`. When `disconnect_on_timeout` is set, a bounded best-effort
/// disconnect is attempted so a failed wait does not leave the tunnel up for the next test.
pub async fn wait_for_tunnel_state_fn(
    rpc: &mut NymProxyClient,
    accept_state_fn: impl Fn(&TunnelState) -> bool,
    timeout: Duration,
    disconnect_on_timeout: bool,
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

    let result = tokio::time::timeout(timeout, async {
        loop {
            match events.next().await {
                Some(Ok(TunnelEvent::NewState(state))) if accept_state_fn(&state) => {
                    log::debug!("Reached expected tunnel state: {state:?}");
                    break Ok(state);
                }
                Some(Ok(event)) => {
                    log::trace!("Ignoring tunnel event: {event:?}");
                    continue;
                }
                Some(Err(status)) => {
                    break Err(Error::Daemon(format!("Failed to get next event: {status}")));
                }
                None => break Err(Error::Daemon(String::from("Lost daemon event stream"))),
            }
        }
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) => {
            // Free the serial mux before follow-up unaries.
            drop(events);

            let poll = poll_tunnel_state(rpc).await;
            if let Ok(ref state) = poll
                && accept_state_fn(state)
            {
                log::warn!(
                    "Tunnel wait timed out after {}s but follow-up poll found expected state: {state:?}",
                    timeout.as_secs()
                );
                return Ok(state.clone());
            }

            let err = format_tunnel_wait_timeout_error(timeout, &poll);
            log::error!("{err}");

            if disconnect_on_timeout {
                best_effort_disconnect(rpc).await;
            }

            Err(Error::Daemon(err))
        }
    }
}

async fn poll_tunnel_state(rpc: &mut NymProxyClient) -> Result<TunnelState, NymClientError> {
    let mut last_err = None;
    for attempt in 1..=TUNNEL_STATE_POLL_ATTEMPTS {
        match rpc.get_tunnel_state().await {
            Ok(state) => return Ok(state),
            Err(err) => {
                log::debug!(
                    "get_tunnel_state poll {attempt}/{TUNNEL_STATE_POLL_ATTEMPTS} failed: {err}"
                );
                last_err = Some(err);
                if attempt < TUNNEL_STATE_POLL_ATTEMPTS {
                    tokio::time::sleep(TUNNEL_STATE_POLL_DELAY).await;
                }
            }
        }
    }
    Err(last_err.expect("TUNNEL_STATE_POLL_ATTEMPTS >= 1"))
}

/// Pure diagnostic for wait timeouts. Never reports bare `None` without an RPC error.
pub(crate) fn format_tunnel_wait_timeout_error(
    timeout: Duration,
    poll: &Result<TunnelState, NymClientError>,
) -> String {
    match poll {
        Ok(state) => format!(
            "Tunnel event listener timed out after {}s. last_observed={state:?} (StillInState)",
            timeout.as_secs()
        ),
        Err(err) => format!(
            "Tunnel event listener timed out after {}s. last_rpc=Err({err}) (RpcFailed); last_observed=<unavailable>",
            timeout.as_secs()
        ),
    }
}

pub(crate) fn format_account_wait_timeout_error(
    timeout: Duration,
    poll: &Result<AccountControllerState, NymClientError>,
) -> String {
    match poll {
        Ok(state) => format!(
            "Account event listener timed out after {}s. last_observed={state:?} (StillInState)",
            timeout.as_secs()
        ),
        Err(err) => format!(
            "Account event listener timed out after {}s. last_rpc=Err({err}) (RpcFailed); last_observed=<unavailable>",
            timeout.as_secs()
        ),
    }
}

pub async fn wait_for_account_state(
    rpc: &mut NymProxyClient,
    expected: AccountControllerState,
) -> Result<AccountControllerState, Error> {
    let timeout = Duration::from_secs(60);
    log::debug!(
        "Waiting for account state: {expected:?} (timeout: {}s)",
        timeout.as_secs()
    );
    wait_for_account_state_fn(rpc, move |state| state.eq(&expected), timeout).await
}

/// Wait for the account to reach a state accepted by `accept_state_fn`, using the daemon event
/// stream. We subscribe to events before checking the current state, so no transitions are missed.
pub async fn wait_for_account_state_fn(
    rpc: &mut NymProxyClient,
    accept_state_fn: impl Fn(&AccountControllerState) -> bool,
    timeout: Duration,
) -> Result<AccountControllerState, Error> {
    let mut events = rpc.listen_to_events().await.map_err(anyhow::Error::msg)?;

    let state = rpc.get_account_state().await.map_err(anyhow::Error::msg)?;

    log::debug!("Current account state: {state:?}");

    if accept_state_fn(&state) {
        return Ok(state);
    }

    let result = tokio::time::timeout(timeout, async {
        loop {
            match events.next().await {
                Some(Ok(TunnelEvent::AccountState(state))) if accept_state_fn(&state) => {
                    log::debug!("Reached expected account state: {state:?}");
                    break Ok(state);
                }
                Some(Ok(event)) => {
                    log::debug!("Ignoring account event: {event:?}");
                    continue;
                }
                Some(Err(status)) => {
                    break Err(Error::Daemon(
                        format!("Failed to get next event: {status}",),
                    ));
                }
                None => break Err(Error::Daemon(String::from("Lost daemon event stream"))),
            }
        }
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) => {
            drop(events);

            let poll = poll_account_state(rpc).await;
            if let Ok(ref state) = poll
                && accept_state_fn(state)
            {
                log::warn!(
                    "Account wait timed out after {}s but follow-up poll found expected state: {state:?}",
                    timeout.as_secs()
                );
                return Ok(state.clone());
            }

            let err = format_account_wait_timeout_error(timeout, &poll);
            log::error!("{err}");
            Err(Error::Daemon(err))
        }
    }
}

async fn poll_account_state(
    rpc: &mut NymProxyClient,
) -> Result<AccountControllerState, NymClientError> {
    let mut last_err = None;
    for attempt in 1..=TUNNEL_STATE_POLL_ATTEMPTS {
        match rpc.get_account_state().await {
            Ok(state) => return Ok(state),
            Err(err) => {
                log::debug!(
                    "get_account_state poll {attempt}/{TUNNEL_STATE_POLL_ATTEMPTS} failed: {err}"
                );
                last_err = Some(err);
                if attempt < TUNNEL_STATE_POLL_ATTEMPTS {
                    tokio::time::sleep(TUNNEL_STATE_POLL_DELAY).await;
                }
            }
        }
    }
    Err(last_err.expect("TUNNEL_STATE_POLL_ATTEMPTS >= 1"))
}

/// useful after tunnel connect/reconnect where the data plane may not be ready
/// immediately after the tunnel state transitions to Connected.
pub async fn resolve_hostname_with_retry(
    rpc: &NymServiceClient,
    hostname: &str,
    timeout: Duration,
) -> anyhow::Result<Vec<SocketAddr>> {
    let hostname = hostname.to_owned();

    let result = tokio::time::timeout(timeout, async {
        loop {
            match rpc.resolve_hostname(hostname.clone()).await {
                Ok(addrs) if !addrs.is_empty() => break addrs,
                Ok(_) => {
                    log::debug!("Got empty result, retrying...");
                }
                Err(e) => {
                    log::debug!("DNS resolution of {hostname} failed: {e}, retrying...");
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    })
    .await;

    match result {
        Ok(addrs) => Ok(addrs),
        Err(_) => {
            let err = format!(
                "DNS resolution of {hostname} timed out after {}s",
                timeout.as_secs(),
            );
            log::error!("{err}");
            anyhow::bail!(err)
        }
    }
}
