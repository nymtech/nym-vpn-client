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
use std::{future::Future, net::SocketAddr, time::Duration};
use test_rpc::NymServiceClient;

/// Bounded best-effort disconnect after a tunnel wait timeout. Must not nest a full
/// `disconnect_and_wait` (that would block the suite for another 40s on a dead serial).
const BEST_EFFORT_DISCONNECT_TIMEOUT: Duration = Duration::from_secs(10);

const TUNNEL_STATE_POLL_ATTEMPTS: u32 = 3;
const TUNNEL_STATE_POLL_DELAY: Duration = Duration::from_millis(500);
const TUNNEL_STATE_POLL_BUDGET: Duration = Duration::from_secs(10);

enum TunnelTimeoutOutcome {
    Recovered(Box<TunnelState>),
    Failed {
        error: String,
        should_disconnect: bool,
    },
}

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
    enforce_tunnel_wait_deadline(
        timeout,
        wait_for_tunnel_state_with_recovery(rpc, accept_state_fn, timeout, disconnect_on_timeout),
    )
    .await
}

async fn wait_for_tunnel_state_with_recovery(
    rpc: &mut NymProxyClient,
    accept_state_fn: impl Fn(&TunnelState) -> bool,
    timeout: Duration,
    disconnect_on_timeout: bool,
) -> Result<TunnelState, Error> {
    let (event_budget, poll_budget) = tunnel_wait_phase_budgets(timeout, disconnect_on_timeout);

    let event_result = tokio::time::timeout(event_budget, async {
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

        loop {
            match events.next().await {
                Some(Ok(TunnelEvent::NewState(state))) if accept_state_fn(&state) => {
                    log::debug!("Reached expected tunnel state: {state:?}");
                    return Ok(state);
                }
                Some(Ok(event)) => {
                    log::trace!("Ignoring tunnel event: {event:?}");
                    continue;
                }
                Some(Err(status)) => {
                    return Err(Error::Daemon(format!("Failed to get next event: {status}")));
                }
                None => return Err(Error::Daemon(String::from("Lost daemon event stream"))),
            }
        }
    })
    .await;

    match event_result {
        Ok(inner) => inner,
        Err(_) => {
            // The timed-out event future is dropped before follow-up unaries,
            // releasing the serial mux.
            let outcome = match tokio::time::timeout(poll_budget, poll_tunnel_state(rpc)).await {
                Ok(poll) => classify_tunnel_timeout(
                    event_budget,
                    &poll,
                    &accept_state_fn,
                    disconnect_on_timeout,
                ),
                Err(_) => TunnelTimeoutOutcome::Failed {
                    error: format!(
                        "Tunnel event phase timed out after {}s. follow-up RPC poll timed out after {}s (RpcFailed); last_observed=<unavailable>",
                        event_budget.as_secs(),
                        poll_budget.as_secs()
                    ),
                    should_disconnect: disconnect_on_timeout,
                },
            };

            match outcome {
                TunnelTimeoutOutcome::Recovered(state) => {
                    log::warn!(
                        "Tunnel event phase timed out after {}s but follow-up poll found expected state: {state:?}",
                        event_budget.as_secs()
                    );
                    Ok(*state)
                }
                TunnelTimeoutOutcome::Failed {
                    error,
                    should_disconnect,
                } => {
                    log::error!("{error}");
                    if should_disconnect {
                        best_effort_disconnect(rpc).await;
                    }
                    Err(Error::Daemon(error))
                }
            }
        }
    }
}

fn tunnel_wait_phase_budgets(
    timeout: Duration,
    disconnect_on_timeout: bool,
) -> (Duration, Duration) {
    let cleanup_budget = if disconnect_on_timeout {
        BEST_EFFORT_DISCONNECT_TIMEOUT.min(timeout)
    } else {
        Duration::ZERO
    };
    let before_cleanup = timeout.saturating_sub(cleanup_budget);
    let poll_budget = TUNNEL_STATE_POLL_BUDGET.min(before_cleanup);
    let event_budget = before_cleanup.saturating_sub(poll_budget);
    (event_budget, poll_budget)
}

async fn enforce_tunnel_wait_deadline<T>(
    timeout: Duration,
    wait: impl Future<Output = Result<T, Error>>,
) -> Result<T, Error> {
    tokio::time::timeout(timeout, wait).await.map_err(|_| {
        Error::Daemon(format!(
            "Tunnel wait exceeded total deadline of {}s",
            timeout.as_secs()
        ))
    })?
}

fn classify_tunnel_timeout(
    timeout: Duration,
    poll: &Result<TunnelState, NymClientError>,
    accept_state_fn: &impl Fn(&TunnelState) -> bool,
    disconnect_on_timeout: bool,
) -> TunnelTimeoutOutcome {
    if let Ok(state) = poll
        && accept_state_fn(state)
    {
        return TunnelTimeoutOutcome::Recovered(Box::new(state.clone()));
    }

    TunnelTimeoutOutcome::Failed {
        error: format_tunnel_wait_timeout_error(timeout, poll),
        should_disconnect: disconnect_on_timeout,
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

#[cfg(test)]
mod tests {
    use super::{
        ExpectedTunnelState, TunnelTimeoutOutcome, classify_tunnel_timeout,
        enforce_tunnel_wait_deadline, format_account_wait_timeout_error,
        format_tunnel_wait_timeout_error, tunnel_wait_params, tunnel_wait_phase_budgets,
    };
    use crate::tests::{WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT, WAIT_FOR_TUNNEL_STATE_TIMEOUT};
    use nym_vpn_lib_types::{AccountControllerState, TunnelState};
    use nym_vpn_proto::rpc_client::Error as NymClientError;
    use std::time::Duration;

    #[test]
    fn connect_wait_uses_extended_timeout_and_cleanup() {
        for state in [
            ExpectedTunnelState::Connected,
            ExpectedTunnelState::Connecting,
        ] {
            assert_eq!(
                tunnel_wait_params(&state),
                (WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT, true)
            );
        }
    }

    #[test]
    fn disconnect_wait_uses_default_timeout_without_cleanup() {
        assert_eq!(
            tunnel_wait_params(&ExpectedTunnelState::Disconnected),
            (WAIT_FOR_TUNNEL_STATE_TIMEOUT, false)
        );
    }

    #[test]
    fn tunnel_timeout_reports_last_observed_state() {
        let message = format_tunnel_wait_timeout_error(
            Duration::from_secs(120),
            &Ok(TunnelState::Disconnected),
        );

        assert!(message.contains("last_observed=Disconnected"));
        assert!(message.contains("StillInState"));
    }

    #[test]
    fn tunnel_timeout_reports_rpc_failure() {
        let message = format_tunnel_wait_timeout_error(
            Duration::from_secs(120),
            &Err(NymClientError::AuthenticationRequired),
        );

        assert!(message.contains("RpcFailed"));
        assert!(message.contains("last_observed=<unavailable>"));
    }

    #[test]
    fn account_timeout_reports_last_observed_state() {
        let message = format_account_wait_timeout_error(
            Duration::from_secs(60),
            &Ok(AccountControllerState::ReadyToConnect),
        );

        assert!(message.contains("last_observed=ReadyToConnect"));
        assert!(message.contains("StillInState"));
    }

    #[test]
    fn late_expected_state_recovers_without_cleanup() {
        let outcome = classify_tunnel_timeout(
            Duration::from_secs(120),
            &Ok(TunnelState::Disconnected),
            &|state| matches!(state, TunnelState::Disconnected),
            true,
        );

        assert!(matches!(
            outcome,
            TunnelTimeoutOutcome::Recovered(state)
                if matches!(*state, TunnelState::Disconnected)
        ));
    }

    #[test]
    fn failed_connect_wait_requests_cleanup() {
        let outcome = classify_tunnel_timeout(
            Duration::from_secs(120),
            &Err(NymClientError::AuthenticationRequired),
            &|state| matches!(state, TunnelState::Connected { .. }),
            true,
        );

        assert!(matches!(
            outcome,
            TunnelTimeoutOutcome::Failed {
                should_disconnect: true,
                ..
            }
        ));
    }

    #[test]
    fn tunnel_phase_budgets_fit_total_deadline() {
        let total = Duration::from_secs(120);
        let (event, poll) = tunnel_wait_phase_budgets(total, true);

        assert_eq!(event, Duration::from_secs(100));
        assert_eq!(poll, Duration::from_secs(10));
        assert_eq!(event + poll + super::BEST_EFFORT_DISCONNECT_TIMEOUT, total);
    }

    #[tokio::test(start_paused = true)]
    async fn tunnel_wait_enforces_total_elapsed_deadline() {
        let timeout = Duration::from_secs(120);
        let started = tokio::time::Instant::now();
        let error = enforce_tunnel_wait_deadline(
            timeout,
            std::future::pending::<Result<(), super::Error>>(),
        )
        .await
        .expect_err("pending wait must hit total deadline");

        assert_eq!(started.elapsed(), timeout);
        assert!(error.to_string().contains("total deadline of 120s"));
    }
}
