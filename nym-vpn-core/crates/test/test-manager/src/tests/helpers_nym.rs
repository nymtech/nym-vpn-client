// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{
    Error, WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT, WAIT_FOR_TUNNEL_STATE_TIMEOUT,
    config_nym::TEST_CONFIG_NYM,
};
use crate::nym_daemon::RpcClientProvider;
use nym_vpn_lib_types::AccountCommandError;
use nym_vpn_proto::rpc_client::{Error as NymClientError, RpcClient as NymProxyClient};
use std::{future::Future, net::SocketAddr, time::Duration};
use test_rpc::{
    NymServiceClient,
    nym_daemon::{
        ObservedAccountState, ObservedAccountStateKind, ObservedTunnelState,
        ObservedTunnelStateKind, WaitOutcome,
    },
};

/// Bounded best-effort disconnect after a tunnel wait timeout. Must not nest a full
/// `disconnect_and_wait` (that would block the suite for another 40s on a dead serial).
const BEST_EFFORT_DISCONNECT_TIMEOUT: Duration = Duration::from_secs(10);

const DAEMON_QUIESCE_SETTLE: Duration = Duration::from_millis(250);

/// Waits for a tunnel-state discriminant on the guest, returning a single reply. A trait so
/// the wait/cleanup orchestration is unit-testable without a live serial link.
trait TunnelWaiter {
    async fn wait_tunnel(
        &self,
        targets: Vec<ObservedTunnelStateKind>,
        timeout: Duration,
    ) -> Result<WaitOutcome<ObservedTunnelState>, Error>;
}

/// Waits for an account-state discriminant on the guest, returning a single reply.
trait AccountWaiter {
    async fn wait_account(
        &self,
        targets: Vec<ObservedAccountStateKind>,
        timeout: Duration,
    ) -> Result<WaitOutcome<ObservedAccountState>, Error>;
}

impl TunnelWaiter for NymServiceClient {
    async fn wait_tunnel(
        &self,
        targets: Vec<ObservedTunnelStateKind>,
        timeout: Duration,
    ) -> Result<WaitOutcome<ObservedTunnelState>, Error> {
        Ok(self
            .wait_for_observed_tunnel_state(targets, timeout)
            .await?)
    }
}

impl AccountWaiter for NymServiceClient {
    async fn wait_account(
        &self,
        targets: Vec<ObservedAccountStateKind>,
        timeout: Duration,
    ) -> Result<WaitOutcome<ObservedAccountState>, Error> {
        Ok(self
            .wait_for_observed_account_state(targets, timeout)
            .await?)
    }
}

trait DisconnectClient {
    async fn disconnect_tunnel(&mut self) -> Result<bool, NymClientError>;

    /// One-shot serial session recreate used only by connect-timeout cleanup.
    async fn recreate_after_disconnect_failure(&mut self) -> Result<bool, Error> {
        Ok(false)
    }
}

/// On-demand serial `disconnect_tunnel` for connect-timeout cleanup. Creates a short-lived
/// gRPC client from the provider (the caller's client is dropped before observe waits).
struct ProviderDisconnectClient<'a> {
    provider: &'a RpcClientProvider,
    rpc: Option<NymProxyClient>,
    recreated: bool,
}

impl ProviderDisconnectClient<'_> {
    fn new(provider: &RpcClientProvider) -> ProviderDisconnectClient<'_> {
        ProviderDisconnectClient {
            provider,
            rpc: None,
            recreated: false,
        }
    }

    async fn ensure_client(&mut self) -> Result<&mut NymProxyClient, NymClientError> {
        if self.rpc.is_none() {
            self.rpc = Some(
                self.provider
                    .new_client_nym()
                    .await
                    .map_err(|error| {
                        NymClientError::Rpc(tonic::Status::unavailable(error.to_string()))
                    })?,
            );
        }
        match self.rpc.as_mut() {
            Some(client) => Ok(client),
            None => Err(NymClientError::Rpc(tonic::Status::internal(
                "ProviderDisconnectClient insert raced to empty",
            ))),
        }
    }

    async fn recreate_once(&mut self) -> Result<bool, Error> {
        if !may_attempt_session_recreate(self.recreated) {
            return Ok(false);
        }
        log::warn!("recreating NymProxyClient after RPC stall");
        self.rpc = Some(self.provider.new_client_nym().await.map_err(Error::Other)?);
        self.recreated = true;
        Ok(true)
    }
}

/// One-shot gate for serial session recreate (testable without a live gRPC client).
pub(crate) fn may_attempt_session_recreate(already_recreated: bool) -> bool {
    !already_recreated
}

impl DisconnectClient for ProviderDisconnectClient<'_> {
    async fn disconnect_tunnel(&mut self) -> Result<bool, NymClientError> {
        let client = self.ensure_client().await?;
        NymProxyClient::disconnect_tunnel(client).await
    }

    async fn recreate_after_disconnect_failure(&mut self) -> Result<bool, Error> {
        self.recreate_once().await
    }
}

#[derive(Debug, PartialEq)]
pub enum ExpectedTunnelState {
    Connected,
    Disconnected,
}

pub const THROTTLE_RETRY_DELAY: Duration = Duration::from_secs(120);

pub async fn login_idempotent(
    runner: &NymServiceClient,
    nym_client: &mut NymProxyClient,
) -> anyhow::Result<()> {
    match runner
        .get_observed_account_state()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get account state: {e}"))?
    {
        ObservedAccountState::ReadyToConnect => {
            return Ok(());
        }
        ObservedAccountState::LoggedOut => {
            store_account_idempotent(nym_client).await?;
        }
        _ => {
            // for other states just wait, AC will either reach ReadyToConnect or we'll timeout.
        }
    }
    wait_for_account_state(runner, ObservedAccountState::ReadyToConnect)
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

pub async fn disconnect_and_wait(
    runner: &NymServiceClient,
    nym_client: NymProxyClient,
    provider: &RpcClientProvider,
) -> Result<NymProxyClient, Error> {
    log::trace!("Disconnecting");
    let mut nym_client = nym_client;
    nym_client.disconnect_tunnel().await?;

    let (_, nym_client) = wait_for_tunnel_state(
        runner,
        nym_client,
        provider,
        ExpectedTunnelState::Disconnected,
    )
    .await?;

    log::trace!("Disconnected");

    Ok(nym_client)
}

/// Best-effort `disconnect_tunnel` with a short deadline. Never waits for Disconnected.
async fn best_effort_disconnect<C>(client: &mut C)
where
    C: DisconnectClient,
{
    match tokio::time::timeout(BEST_EFFORT_DISCONNECT_TIMEOUT, client.disconnect_tunnel()).await {
        Ok(Ok(_)) => {
            log::info!("Best-effort disconnect_tunnel succeeded");
            return;
        }
        Ok(Err(err)) => log::warn!("Best-effort disconnect_tunnel failed: {err}"),
        Err(_) => log::warn!(
            "Best-effort disconnect_tunnel timed out after {}s",
            BEST_EFFORT_DISCONNECT_TIMEOUT.as_secs()
        ),
    }

    match client.recreate_after_disconnect_failure().await {
        Ok(true) => {
            match tokio::time::timeout(BEST_EFFORT_DISCONNECT_TIMEOUT, client.disconnect_tunnel())
                .await
            {
                Ok(Ok(_)) => log::info!("Best-effort disconnect_tunnel succeeded after recreate"),
                Ok(Err(err)) => {
                    log::warn!("Best-effort disconnect_tunnel failed after recreate: {err}")
                }
                Err(_) => log::warn!(
                    "Best-effort disconnect_tunnel timed out after recreate ({}s)",
                    BEST_EFFORT_DISCONNECT_TIMEOUT.as_secs()
                ),
            }
        }
        Ok(false) => {}
        Err(err) => log::warn!("Best-effort serial session recreate failed: {err}"),
    }
}

pub async fn wait_for_tunnel_state(
    runner: &NymServiceClient,
    nym_client: NymProxyClient,
    provider: &RpcClientProvider,
    expected: ExpectedTunnelState,
) -> Result<(ObservedTunnelState, NymProxyClient), Error> {
    let (timeout, disconnect_on_timeout) = tunnel_wait_params(&expected);
    log::debug!(
        "Waiting for tunnel state: {expected:?} (timeout: {}s)",
        timeout.as_secs()
    );
    // Drop the live HTTP/2-over-serial session before the guest observe wait. An idle-but-open
    // DaemonRpc channel keeps competing for the 115200 baud mux and can HOL-block the single
    // WaitOutcome reply after Connected (Debian12 E2E).
    log::info!("quiescing serial DaemonRpc before observe wait");
    drop(nym_client);
    settle_daemon_rpc_quiesce().await;

    let observed = wait_for_tunnel_state_fn(
        runner,
        provider,
        vec![tunnel_target(&expected)],
        timeout,
        disconnect_on_timeout,
    )
    .await;
    let client = provider.new_client_nym().await.map_err(Error::Other);
    match merge_tunnel_wait_and_client(observed, client) {
        Ok(pair) => Ok(pair),
        Err((error, client)) => {
            // Recreate still ran on the observe-failure path so suite/provider cleanup can open a
            // fresh DaemonRpc session; public Err cannot carry the client without a signature break.
            drop(client);
            Err(error)
        }
    }
}

/// Settle the serial mux, run the guest observe wait, then always attempt client recreate.
pub(crate) async fn quiesce_observe_and_recreate<O, R, C>(
    observe: O,
    recreate: R,
) -> Result<(ObservedTunnelState, C), (Error, Option<C>)>
where
    O: std::future::Future<Output = Result<ObservedTunnelState, Error>>,
    R: std::future::Future<Output = Result<C, Error>>,
{
    settle_daemon_rpc_quiesce().await;
    let observed = observe.await;
    let client = recreate.await;
    merge_tunnel_wait_and_client(observed, client)
}

pub(crate) async fn settle_daemon_rpc_quiesce() {
    tokio::time::sleep(DAEMON_QUIESCE_SETTLE).await;
}

/// Prefer the observe outcome; always surface a recreated client on wait failure when recreate Ok.
pub(crate) fn merge_tunnel_wait_and_client<C>(
    observed: Result<ObservedTunnelState, Error>,
    client: Result<C, Error>,
) -> Result<(ObservedTunnelState, C), (Error, Option<C>)> {
    match (observed, client) {
        (Ok(state), Ok(client)) => Ok((state, client)),
        (Err(wait_error), Ok(client)) => Err((wait_error, Some(client))),
        (Ok(_state), Err(recreate_error)) => Err((recreate_error, None)),
        (Err(wait_error), Err(_recreate_error)) => Err((wait_error, None)),
    }
}

/// Connect waits get a longer timeout and disconnect-on-timeout; other waits do not.
pub(crate) fn tunnel_wait_params(expected: &ExpectedTunnelState) -> (Duration, bool) {
    match expected {
        ExpectedTunnelState::Connected => (WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT, true),
        ExpectedTunnelState::Disconnected => (WAIT_FOR_TUNNEL_STATE_TIMEOUT, false),
    }
}

/// Payload-insensitive discriminant selector for a given expected tunnel state.
pub(crate) fn tunnel_target(expected: &ExpectedTunnelState) -> ObservedTunnelStateKind {
    match expected {
        ExpectedTunnelState::Connected => ObservedTunnelStateKind::Connected,
        ExpectedTunnelState::Disconnected => ObservedTunnelStateKind::Disconnected,
    }
}

/// Wait for a tunnel discriminant via a single guest tarpc call (local UDS on the VM). The
/// guest polls its daemon locally and returns one reply, so no per-poll frame (and no large
/// `ConnectionData` gRPC payload) crosses the serial link during the Connected wait.
///
/// Callers must drop any live serial `NymProxyClient` before this (see [`wait_for_tunnel_state`])
/// so DaemonRpc HTTP/2 cannot HOL the WaitOutcome on the shared mux.
pub async fn wait_for_tunnel_state_fn(
    runner: &NymServiceClient,
    provider: &RpcClientProvider,
    targets: Vec<ObservedTunnelStateKind>,
    timeout: Duration,
    disconnect_on_timeout: bool,
) -> Result<ObservedTunnelState, Error> {
    let mut disconnect_client = ProviderDisconnectClient::new(provider);
    enforce_tunnel_wait_deadline(
        timeout,
        run_tunnel_wait(
            runner,
            &mut disconnect_client,
            targets,
            timeout,
            disconnect_on_timeout,
        ),
    )
    .await
}

async fn run_tunnel_wait<W, D>(
    waiter: &W,
    disconnect_client: &mut D,
    targets: Vec<ObservedTunnelStateKind>,
    timeout: Duration,
    disconnect_on_timeout: bool,
) -> Result<ObservedTunnelState, Error>
where
    W: TunnelWaiter,
    D: DisconnectClient,
{
    let budget = tunnel_wait_budget(timeout, disconnect_on_timeout);
    log::info!(
        "tunnel wait: calling guest wait_for_observed (budget={}s, disconnect_on_timeout={disconnect_on_timeout})",
        budget.as_secs()
    );

    let result = match waiter.wait_tunnel(targets, budget).await {
        Ok(outcome) => {
            log::info!("tunnel wait: guest replied with {outcome:?}");
            tunnel_outcome_to_result(outcome, budget)
        }
        Err(error) => {
            log::error!("tunnel wait: guest RPC failed: {error}");
            Err(rpc_wait_error("tunnel", error))
        }
    };

    if let Err(error) = &result {
        log::error!("{error}");
        if disconnect_on_timeout {
            best_effort_disconnect(disconnect_client).await;
        }
    }

    result
}

/// Reserve a cleanup window inside the total deadline for connect waits, so a best-effort
/// disconnect still fits before the outer deadline fires.
fn tunnel_wait_budget(timeout: Duration, disconnect_on_timeout: bool) -> Duration {
    let cleanup_budget = if disconnect_on_timeout {
        BEST_EFFORT_DISCONNECT_TIMEOUT.min(timeout)
    } else {
        Duration::ZERO
    };
    timeout.saturating_sub(cleanup_budget)
}

fn tunnel_outcome_to_result(
    outcome: WaitOutcome<ObservedTunnelState>,
    budget: Duration,
) -> Result<ObservedTunnelState, Error> {
    match outcome {
        WaitOutcome::Reached(state) => Ok(state),
        WaitOutcome::TimedOut { last_observed } => Err(wait_timeout_error(
            "tunnel",
            budget,
            last_observed.as_ref().map(|state| format!("{state:?}")),
        )),
    }
}

fn wait_timeout_error(state_name: &str, budget: Duration, last_observed: Option<String>) -> Error {
    Error::Daemon(format!(
        "{state_name} state wait timed out after {}s; last_observed={}",
        budget.as_secs(),
        last_observed.unwrap_or_else(|| "<unavailable>".to_owned()),
    ))
}

fn rpc_wait_error(state_name: &str, error: Error) -> Error {
    Error::Daemon(format!("{state_name} state wait RPC failed: {error}"))
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

pub async fn wait_for_account_state(
    runner: &NymServiceClient,
    expected: ObservedAccountState,
) -> Result<ObservedAccountState, Error> {
    let timeout = Duration::from_secs(60);
    log::debug!(
        "Waiting for account state: {expected:?} (timeout: {}s)",
        timeout.as_secs()
    );
    run_account_wait(runner, vec![account_target(&expected)], timeout).await
}

/// Payload-insensitive discriminant selector for a given expected account state.
pub(crate) fn account_target(expected: &ObservedAccountState) -> ObservedAccountStateKind {
    match expected {
        ObservedAccountState::Offline => ObservedAccountStateKind::Offline,
        ObservedAccountState::Syncing => ObservedAccountStateKind::Syncing,
        ObservedAccountState::LoggedOut => ObservedAccountStateKind::LoggedOut,
        ObservedAccountState::ReadyToConnect => ObservedAccountStateKind::ReadyToConnect,
        ObservedAccountState::Decentralised => ObservedAccountStateKind::Decentralised,
        ObservedAccountState::PendingSubscription => ObservedAccountStateKind::PendingSubscription,
        ObservedAccountState::Error(_) => ObservedAccountStateKind::Error,
    }
}

/// Wait for an account discriminant via a single guest tarpc call (local UDS).
async fn run_account_wait<W>(
    waiter: &W,
    targets: Vec<ObservedAccountStateKind>,
    timeout: Duration,
) -> Result<ObservedAccountState, Error>
where
    W: AccountWaiter,
{
    log::info!(
        "account wait: calling guest wait_for_observed (timeout={}s)",
        timeout.as_secs()
    );
    match waiter.wait_account(targets, timeout).await {
        Ok(outcome) => {
            log::info!("account wait: guest replied with {outcome:?}");
            match outcome {
                WaitOutcome::Reached(state) => Ok(state),
                WaitOutcome::TimedOut { last_observed } => Err(wait_timeout_error(
                    "account",
                    timeout,
                    last_observed.as_ref().map(|state| format!("{state:?}")),
                )),
            }
        }
        Err(error) => {
            log::error!("account wait: guest RPC failed: {error}");
            Err(rpc_wait_error("account", error))
        }
    }
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
        AccountWaiter, DisconnectClient, ExpectedTunnelState, TunnelWaiter, account_target,
        enforce_tunnel_wait_deadline, merge_tunnel_wait_and_client, quiesce_observe_and_recreate,
        run_account_wait, run_tunnel_wait, settle_daemon_rpc_quiesce, tunnel_target,
        tunnel_wait_budget, tunnel_wait_params,
    };
    use crate::tests::{Error, WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT, WAIT_FOR_TUNNEL_STATE_TIMEOUT};
    use futures::StreamExt;
    use nym_vpn_proto::rpc_client::Error as NymClientError;
    use std::time::Duration;
    use test_rpc::nym_daemon::{
        ObservedAccountState, ObservedAccountStateKind, ObservedTunnelState,
        ObservedTunnelStateKind, ObservedTunnelType, WaitOutcome,
    };

    fn rpc_failed() -> Error {
        Error::NymManagementInterface(NymClientError::AuthenticationRequired)
    }

    /// Canned single-reply behavior for a faked guest tunnel wait.
    enum TunnelReply {
        Reached(ObservedTunnelState),
        TimedOut(Option<ObservedTunnelState>),
        RpcError,
    }

    struct FakeTunnelWaiter {
        reply: TunnelReply,
    }

    impl TunnelWaiter for FakeTunnelWaiter {
        async fn wait_tunnel(
            &self,
            _targets: Vec<ObservedTunnelStateKind>,
            _timeout: Duration,
        ) -> Result<WaitOutcome<ObservedTunnelState>, Error> {
            match &self.reply {
                TunnelReply::Reached(state) => Ok(WaitOutcome::Reached(state.clone())),
                TunnelReply::TimedOut(last) => Ok(WaitOutcome::TimedOut {
                    last_observed: last.clone(),
                }),
                TunnelReply::RpcError => Err(rpc_failed()),
            }
        }
    }

    struct FakeAccountWaiter {
        reply: Result<WaitOutcome<ObservedAccountState>, ()>,
    }

    impl AccountWaiter for FakeAccountWaiter {
        async fn wait_account(
            &self,
            _targets: Vec<ObservedAccountStateKind>,
            _timeout: Duration,
        ) -> Result<WaitOutcome<ObservedAccountState>, Error> {
            match &self.reply {
                Ok(outcome) => Ok(outcome.clone()),
                Err(()) => Err(rpc_failed()),
            }
        }
    }

    struct FakeDisconnectClient {
        disconnects: usize,
    }

    impl DisconnectClient for FakeDisconnectClient {
        async fn disconnect_tunnel(&mut self) -> Result<bool, NymClientError> {
            self.disconnects += 1;
            Ok(true)
        }
    }

    #[test]
    fn tunnel_target_selects_discriminant() {
        assert_eq!(
            tunnel_target(&ExpectedTunnelState::Connected),
            ObservedTunnelStateKind::Connected
        );
        assert_eq!(
            tunnel_target(&ExpectedTunnelState::Disconnected),
            ObservedTunnelStateKind::Disconnected
        );
    }

    #[test]
    fn account_target_selects_discriminant() {
        assert_eq!(
            account_target(&ObservedAccountState::ReadyToConnect),
            ObservedAccountStateKind::ReadyToConnect
        );
        assert_eq!(
            account_target(&ObservedAccountState::Error("ignored".into())),
            ObservedAccountStateKind::Error
        );
    }

    #[test]
    fn connect_wait_uses_extended_timeout_and_cleanup() {
        assert_eq!(
            tunnel_wait_params(&ExpectedTunnelState::Connected),
            (WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT, true)
        );
    }

    #[test]
    fn disconnect_wait_uses_default_timeout_without_cleanup() {
        assert_eq!(
            tunnel_wait_params(&ExpectedTunnelState::Disconnected),
            (WAIT_FOR_TUNNEL_STATE_TIMEOUT, false)
        );
    }

    #[test]
    fn connect_reserves_cleanup_inside_total_deadline() {
        assert_eq!(
            tunnel_wait_budget(WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT, true),
            WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT - super::BEST_EFFORT_DISCONNECT_TIMEOUT
        );
        assert_eq!(
            tunnel_wait_budget(WAIT_FOR_TUNNEL_STATE_TIMEOUT, false),
            WAIT_FOR_TUNNEL_STATE_TIMEOUT
        );
    }

    #[test]
    fn session_recreate_permit_is_one_shot() {
        assert!(super::may_attempt_session_recreate(false));
        assert!(!super::may_attempt_session_recreate(true));
    }

    #[tokio::test]
    async fn reached_tunnel_state_returns_without_cleanup() {
        let waiter = FakeTunnelWaiter {
            reply: TunnelReply::Reached(ObservedTunnelState::Connected {
                tunnel_type: ObservedTunnelType::Wireguard,
            }),
        };
        let mut disconnect = FakeDisconnectClient { disconnects: 0 };

        let state = run_tunnel_wait(
            &waiter,
            &mut disconnect,
            vec![ObservedTunnelStateKind::Connected],
            Duration::from_secs(2),
            true,
        )
        .await
        .expect("reached state is a success");

        assert!(matches!(
            state,
            ObservedTunnelState::Connected {
                tunnel_type: ObservedTunnelType::Wireguard
            }
        ));
        assert_eq!(disconnect.disconnects, 0);
    }

    #[tokio::test]
    async fn tunnel_timeout_reports_last_observed_and_cleans_up() {
        let waiter = FakeTunnelWaiter {
            reply: TunnelReply::TimedOut(Some(ObservedTunnelState::Connecting)),
        };
        let mut disconnect = FakeDisconnectClient { disconnects: 0 };

        let error = run_tunnel_wait(
            &waiter,
            &mut disconnect,
            vec![ObservedTunnelStateKind::Connected],
            Duration::from_secs(2),
            true,
        )
        .await
        .expect_err("a timeout outcome must surface as an error");

        assert!(error.to_string().contains("tunnel state wait timed out"));
        assert!(error.to_string().contains("last_observed=Connecting"));
        assert_eq!(disconnect.disconnects, 1);
    }

    #[tokio::test]
    async fn tunnel_timeout_without_cleanup_skips_disconnect() {
        let waiter = FakeTunnelWaiter {
            reply: TunnelReply::TimedOut(None),
        };
        let mut disconnect = FakeDisconnectClient { disconnects: 0 };

        run_tunnel_wait(
            &waiter,
            &mut disconnect,
            vec![ObservedTunnelStateKind::Disconnected],
            Duration::from_secs(2),
            false,
        )
        .await
        .expect_err("timeout is an error");

        assert_eq!(disconnect.disconnects, 0);
    }

    #[tokio::test]
    async fn tunnel_rpc_error_cleans_up_on_connect_wait() {
        let waiter = FakeTunnelWaiter {
            reply: TunnelReply::RpcError,
        };
        let mut disconnect = FakeDisconnectClient { disconnects: 0 };

        let error = run_tunnel_wait(
            &waiter,
            &mut disconnect,
            vec![ObservedTunnelStateKind::Connected],
            Duration::from_secs(2),
            true,
        )
        .await
        .expect_err("rpc failure must surface as an error");

        assert!(error.to_string().contains("tunnel state wait RPC failed"));
        assert_eq!(disconnect.disconnects, 1);
    }

    #[tokio::test]
    async fn reached_account_state_returns_ok() {
        let waiter = FakeAccountWaiter {
            reply: Ok(WaitOutcome::Reached(ObservedAccountState::ReadyToConnect)),
        };

        let state = run_account_wait(
            &waiter,
            vec![ObservedAccountStateKind::ReadyToConnect],
            Duration::from_secs(2),
        )
        .await
        .expect("reached state is a success");

        assert!(matches!(state, ObservedAccountState::ReadyToConnect));
    }

    #[tokio::test]
    async fn account_timeout_reports_last_observed() {
        let waiter = FakeAccountWaiter {
            reply: Ok(WaitOutcome::TimedOut {
                last_observed: Some(ObservedAccountState::Syncing),
            }),
        };

        let error = run_account_wait(
            &waiter,
            vec![ObservedAccountStateKind::ReadyToConnect],
            Duration::from_secs(2),
        )
        .await
        .expect_err("a timeout outcome must surface as an error");

        assert!(error.to_string().contains("account state wait timed out"));
        assert!(error.to_string().contains("last_observed=Syncing"));
    }

    #[tokio::test(start_paused = true)]
    async fn tunnel_wait_enforces_total_elapsed_deadline() {
        let timeout = Duration::from_secs(120);
        let started = tokio::time::Instant::now();
        let error =
            enforce_tunnel_wait_deadline(timeout, std::future::pending::<Result<(), Error>>())
                .await
                .expect_err("pending wait must hit total deadline");

        assert_eq!(started.elapsed(), timeout);
        assert!(error.to_string().contains("total deadline of 120s"));
    }

    #[tokio::test(start_paused = true)]
    async fn daemon_quiesce_settle_advances_paused_clock() {
        let started = tokio::time::Instant::now();
        settle_daemon_rpc_quiesce().await;
        assert_eq!(started.elapsed(), super::DAEMON_QUIESCE_SETTLE);
    }

    #[test]
    fn merge_tunnel_wait_returns_client_on_observe_error() {
        let observed: Result<ObservedTunnelState, Error> = Err(Error::Daemon("observe failed".into()));
        let client: Result<&str, Error> = Ok("recreated");

        let err = merge_tunnel_wait_and_client(observed, client)
            .expect_err("observe failure must surface");
        assert!(err.0.to_string().contains("observe failed"));
        assert_eq!(err.1, Some("recreated"));
    }

    #[test]
    fn merge_tunnel_wait_success_pairs_state_and_client() {
        let observed = Ok(ObservedTunnelState::Disconnected);
        let client: Result<&str, Error> = Ok("recreated");

        let (state, got) =
            merge_tunnel_wait_and_client(observed, client).expect("success pairs both");
        assert!(matches!(state, ObservedTunnelState::Disconnected));
        assert_eq!(got, "recreated");
    }

    #[test]
    fn merge_tunnel_wait_prefers_observe_error_when_recreate_also_fails() {
        let observed: Result<ObservedTunnelState, Error> = Err(Error::Daemon("observe failed".into()));
        let client: Result<&str, Error> = Err(Error::Daemon("recreate failed".into()));

        let err = merge_tunnel_wait_and_client(observed, client)
            .expect_err("combined failure must surface");
        assert!(err.0.to_string().contains("observe failed"));
        assert!(err.1.is_none());
    }

    #[tokio::test(start_paused = true)]
    async fn quiesce_observe_recreates_client_after_observe_error() {
        let started = tokio::time::Instant::now();
        let mut recreate_calls = 0usize;

        let result = quiesce_observe_and_recreate(
            async {
                Err(Error::Daemon("guest timed out".into()))
            },
            async {
                recreate_calls += 1;
                Ok("replacement-client")
            },
        )
        .await;

        assert_eq!(started.elapsed(), super::DAEMON_QUIESCE_SETTLE);
        assert_eq!(recreate_calls, 1);
        let (error, client) = result.expect_err("observe error must propagate");
        assert!(error.to_string().contains("guest timed out"));
        assert_eq!(client, Some("replacement-client"));
    }

    #[tokio::test(start_paused = true)]
    async fn quiesce_observe_recreates_client_after_observe_success() {
        let started = tokio::time::Instant::now();
        let mut recreate_calls = 0usize;

        let (state, client) = quiesce_observe_and_recreate(
            async { Ok(ObservedTunnelState::Connected {
                tunnel_type: ObservedTunnelType::Wireguard,
            }) },
            async {
                recreate_calls += 1;
                Ok("replacement-client")
            },
        )
        .await
        .expect("success path returns state and client");

        assert_eq!(started.elapsed(), super::DAEMON_QUIESCE_SETTLE);
        assert_eq!(recreate_calls, 1);
        assert!(matches!(
            state,
            ObservedTunnelState::Connected {
                tunnel_type: ObservedTunnelType::Wireguard
            }
        ));
        assert_eq!(client, "replacement-client");
    }

    #[tokio::test(start_paused = true)]
    async fn provider_dangling_create_is_requested_on_new_client_nym() {
        let (provider, mut rx) = crate::nym_daemon::RpcClientProvider::dangling_for_tests();
        let create = tokio::spawn(async move { provider.new_client_nym().await });
        let channel = tokio::time::timeout(Duration::from_secs(1), rx.next())
            .await
            .expect("disconnect cleanup recreate must request a management duplex")
            .expect("channel remains open");
        drop(channel);
        create.abort();
    }
}
