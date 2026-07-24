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
use std::{fmt::Debug, future::Future, net::SocketAddr, time::Duration};
use test_rpc::{
    NymServiceClient,
    nym_daemon::{ObservedAccountState, ObservedTunnelState},
};

/// Bounded best-effort disconnect after a tunnel wait timeout. Must not nest a full
/// `disconnect_and_wait` (that would block the suite for another 40s on a dead serial).
const BEST_EFFORT_DISCONNECT_TIMEOUT: Duration = Duration::from_secs(10);

const TUNNEL_STATE_POLL_DELAY: Duration = Duration::from_millis(500);
const TUNNEL_STATE_RPC_TIMEOUT: Duration = Duration::from_secs(30);

trait StateClient<S> {
    async fn read_state(&mut self) -> Result<S, Error>;
}

trait DisconnectClient {
    async fn disconnect_tunnel(&mut self) -> Result<bool, NymClientError>;

    /// One-shot serial session recreate used only by connect-timeout cleanup.
    async fn recreate_after_disconnect_failure(&mut self) -> Result<bool, Error> {
        Ok(false)
    }
}

/// Guest tarpc observer: reads tunnel/account discriminants via local UDS on the VM.
struct RunnerObservedClient<'a> {
    runner: &'a NymServiceClient,
}

impl StateClient<ObservedTunnelState> for RunnerObservedClient<'_> {
    async fn read_state(&mut self) -> Result<ObservedTunnelState, Error> {
        self.runner
            .get_observed_tunnel_state()
            .await
            .map_err(Error::from)
    }
}

impl StateClient<ObservedAccountState> for RunnerObservedClient<'_> {
    async fn read_state(&mut self) -> Result<ObservedAccountState, Error> {
        self.runner
            .get_observed_account_state()
            .await
            .map_err(Error::from)
    }
}

/// Serial `disconnect_tunnel` for connect-timeout cleanup (commands stay on serial gRPC).
struct SerialDisconnectClient<'a> {
    rpc: &'a mut NymProxyClient,
    provider: &'a RpcClientProvider,
    recreated: bool,
}

impl SerialDisconnectClient<'_> {
    async fn recreate_once(&mut self) -> Result<bool, Error> {
        if !may_attempt_session_recreate(self.recreated) {
            return Ok(false);
        }
        log::warn!("recreating NymProxyClient after RPC stall");
        *self.rpc = self.provider.new_client_nym().await.map_err(Error::Other)?;
        self.recreated = true;
        Ok(true)
    }
}

/// One-shot gate for serial session recreate (testable without a live gRPC client).
pub(crate) fn may_attempt_session_recreate(already_recreated: bool) -> bool {
    !already_recreated
}

impl DisconnectClient for SerialDisconnectClient<'_> {
    async fn disconnect_tunnel(&mut self) -> Result<bool, NymClientError> {
        NymProxyClient::disconnect_tunnel(self.rpc).await
    }

    async fn recreate_after_disconnect_failure(&mut self) -> Result<bool, Error> {
        self.recreate_once().await
    }
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

impl From<&ObservedTunnelState> for ExpectedTunnelState {
    fn from(value: &ObservedTunnelState) -> Self {
        match value {
            ObservedTunnelState::Connected { .. } => ExpectedTunnelState::Connected,
            ObservedTunnelState::Disconnected => ExpectedTunnelState::Disconnected,
            ObservedTunnelState::Connecting => ExpectedTunnelState::Connecting,
            ObservedTunnelState::Disconnecting => ExpectedTunnelState::Disconnecting,
            ObservedTunnelState::Offline => ExpectedTunnelState::Offline,
            ObservedTunnelState::Error(reason) => ExpectedTunnelState::Error(reason.clone()),
        }
    }
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
    nym_client: &mut NymProxyClient,
    provider: &RpcClientProvider,
) -> Result<(), Error> {
    log::trace!("Disconnecting");
    nym_client.disconnect_tunnel().await?;

    wait_for_tunnel_state_fn(
        runner,
        nym_client,
        provider,
        |state| matches!(state, ObservedTunnelState::Disconnected),
        WAIT_FOR_TUNNEL_STATE_TIMEOUT,
        false,
    )
    .await?;

    log::trace!("Disconnected");

    Ok(())
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
    nym_client: &mut NymProxyClient,
    provider: &RpcClientProvider,
    expected: ExpectedTunnelState,
) -> Result<ObservedTunnelState, Error> {
    let (timeout, disconnect_on_timeout) = tunnel_wait_params(&expected);
    log::debug!(
        "Waiting for tunnel state: {expected:?} (timeout: {}s)",
        timeout.as_secs()
    );
    wait_for_tunnel_state_fn(
        runner,
        nym_client,
        provider,
        move |state| ExpectedTunnelState::from(state) == expected,
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

/// Wait for the tunnel discriminant via guest tarpc (local UDS), not serial-forwarded gRPC.
pub async fn wait_for_tunnel_state_fn(
    runner: &NymServiceClient,
    nym_client: &mut NymProxyClient,
    provider: &RpcClientProvider,
    accept_state_fn: impl Fn(&ObservedTunnelState) -> bool,
    timeout: Duration,
    disconnect_on_timeout: bool,
) -> Result<ObservedTunnelState, Error> {
    let mut observer = RunnerObservedClient { runner };
    let mut disconnect_client = SerialDisconnectClient {
        rpc: nym_client,
        provider,
        recreated: false,
    };
    enforce_tunnel_wait_deadline(
        timeout,
        wait_for_tunnel_state_with_polling(
            &mut observer,
            &mut disconnect_client,
            accept_state_fn,
            timeout,
            disconnect_on_timeout,
        ),
    )
    .await
}

async fn wait_for_tunnel_state_with_polling<R, D>(
    reader: &mut R,
    disconnect_client: &mut D,
    accept_state_fn: impl Fn(&ObservedTunnelState) -> bool,
    timeout: Duration,
    disconnect_on_timeout: bool,
) -> Result<ObservedTunnelState, Error>
where
    R: StateClient<ObservedTunnelState>,
    D: DisconnectClient,
{
    let polling_budget = tunnel_polling_budget(timeout, disconnect_on_timeout);

    match poll_tunnel_state_until(reader, accept_state_fn, polling_budget).await {
        Ok(state) => Ok(state),
        Err(error) => {
            log::error!("{error}");
            if disconnect_on_timeout {
                best_effort_disconnect(disconnect_client).await;
            }
            Err(error)
        }
    }
}

fn tunnel_polling_budget(timeout: Duration, disconnect_on_timeout: bool) -> Duration {
    let cleanup_budget = if disconnect_on_timeout {
        BEST_EFFORT_DISCONNECT_TIMEOUT.min(timeout)
    } else {
        Duration::ZERO
    };
    timeout.saturating_sub(cleanup_budget)
}

async fn poll_tunnel_state_until<R>(
    reader: &mut R,
    accept_state_fn: impl Fn(&ObservedTunnelState) -> bool,
    timeout: Duration,
) -> Result<ObservedTunnelState, Error>
where
    R: StateClient<ObservedTunnelState>,
{
    poll_state_until(reader, accept_state_fn, timeout, "tunnel").await
}

async fn poll_state_until<R, S>(
    reader: &mut R,
    accept_state_fn: impl Fn(&S) -> bool,
    timeout: Duration,
    state_name: &str,
) -> Result<S, Error>
where
    R: StateClient<S>,
    S: Debug,
{
    let deadline = tokio::time::Instant::now() + timeout;
    let mut last_observed = None;
    let mut last_rpc_error = None;
    let mut attempt: u64 = 0;

    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }

        attempt += 1;
        let rpc_timeout = TUNNEL_STATE_RPC_TIMEOUT.min(remaining);
        let sent_at = time::OffsetDateTime::now_utc();
        log::info!("poll[{state_name}#{attempt}] sending get_{state_name}_state at {sent_at}");
        match tokio::time::timeout(rpc_timeout, reader.read_state()).await {
            Ok(Ok(state)) => {
                log::info!(
                    "poll[{state_name}#{attempt}] received response at {}: {state:?}",
                    time::OffsetDateTime::now_utc()
                );
                if accept_state_fn(&state) {
                    log::debug!("Reached expected {state_name} state: {state:?}");
                    return Ok(state);
                }
                last_observed = Some(state);
                last_rpc_error = None;
            }
            Ok(Err(error)) => {
                log::info!(
                    "poll[{state_name}#{attempt}] rpc error at {}: {error}",
                    time::OffsetDateTime::now_utc()
                );
                last_rpc_error = Some(format!("Err({error}) (RpcFailed)"));
            }
            Err(_) => {
                log::info!(
                    "poll[{state_name}#{attempt}] client-side timeout at {} after {}s (no response)",
                    time::OffsetDateTime::now_utc(),
                    rpc_timeout.as_secs()
                );
                last_rpc_error = Some(format!(
                    "TimedOut({}s) (RpcTimedOut)",
                    rpc_timeout.as_secs()
                ));
            }
        }

        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        tokio::time::sleep(TUNNEL_STATE_POLL_DELAY.min(remaining)).await;
    }

    let last_observed = last_observed
        .as_ref()
        .map_or_else(|| "<unavailable>".to_owned(), |state| format!("{state:?}"));
    let last_rpc = last_rpc_error.unwrap_or_else(|| "Ok (StillInState)".to_owned());
    Err(Error::Daemon(format!(
        "{state_name} state polling timed out after {}s. last_rpc={last_rpc}; last_observed={last_observed}",
        timeout.as_secs()
    )))
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
    wait_for_account_state_fn(runner, move |state| state.eq(&expected), timeout).await
}

/// Wait for account readiness via guest tarpc (local UDS).
pub async fn wait_for_account_state_fn(
    runner: &NymServiceClient,
    accept_state_fn: impl Fn(&ObservedAccountState) -> bool,
    timeout: Duration,
) -> Result<ObservedAccountState, Error> {
    let mut observer = RunnerObservedClient { runner };
    poll_account_state_until(&mut observer, accept_state_fn, timeout).await
}

async fn poll_account_state_until<R>(
    reader: &mut R,
    accept_state_fn: impl Fn(&ObservedAccountState) -> bool,
    timeout: Duration,
) -> Result<ObservedAccountState, Error>
where
    R: StateClient<ObservedAccountState>,
{
    poll_state_until(reader, accept_state_fn, timeout, "account").await
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
        DisconnectClient, ExpectedTunnelState, StateClient, enforce_tunnel_wait_deadline,
        poll_account_state_until, poll_tunnel_state_until, tunnel_polling_budget,
        tunnel_wait_params, wait_for_tunnel_state_with_polling,
    };
    use crate::tests::{Error, WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT, WAIT_FOR_TUNNEL_STATE_TIMEOUT};
    use futures::StreamExt;
    use nym_vpn_proto::rpc_client::Error as NymClientError;
    use std::{collections::VecDeque, future::pending, time::Duration};
    use test_rpc::nym_daemon::{ObservedAccountState, ObservedTunnelState, ObservedTunnelType};

    enum FakeRead {
        Result(Box<Result<ObservedTunnelState, Error>>),
        Pending,
    }

    struct FakeTunnelStateReader {
        reads: VecDeque<FakeRead>,
    }

    impl FakeTunnelStateReader {
        fn new(reads: impl IntoIterator<Item = FakeRead>) -> Self {
            Self {
                reads: reads.into_iter().collect(),
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

    struct FakeAccountStateReader {
        reads: VecDeque<Result<ObservedAccountState, Error>>,
    }

    impl FakeAccountStateReader {
        fn new(reads: impl IntoIterator<Item = Result<ObservedAccountState, Error>>) -> Self {
            Self {
                reads: reads.into_iter().collect(),
            }
        }
    }

    impl StateClient<ObservedAccountState> for FakeAccountStateReader {
        async fn read_state(&mut self) -> Result<ObservedAccountState, Error> {
            self.reads
                .pop_front()
                .unwrap_or(Ok(ObservedAccountState::ReadyToConnect))
        }
    }

    impl StateClient<ObservedTunnelState> for FakeTunnelStateReader {
        async fn read_state(&mut self) -> Result<ObservedTunnelState, Error> {
            match self
                .reads
                .pop_front()
                .unwrap_or(FakeRead::Result(Box::new(Ok(
                    ObservedTunnelState::Disconnected,
                )))) {
                FakeRead::Result(result) => *result,
                FakeRead::Pending => pending().await,
            }
        }
    }

    fn rpc_failed() -> Error {
        Error::NymManagementInterface(NymClientError::AuthenticationRequired)
    }

    #[test]
    fn observed_tunnel_maps_to_expected() {
        assert_eq!(
            ExpectedTunnelState::from(&ObservedTunnelState::Connected {
                tunnel_type: ObservedTunnelType::Wireguard
            }),
            ExpectedTunnelState::Connected
        );
        assert_eq!(
            ExpectedTunnelState::from(&ObservedTunnelState::Error("x".into())),
            ExpectedTunnelState::Error("x".into())
        );
        assert_eq!(
            ExpectedTunnelState::from(&ObservedTunnelState::Offline),
            ExpectedTunnelState::Offline
        );
    }

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
    fn connect_polling_reserves_cleanup_inside_total_deadline() {
        assert_eq!(
            tunnel_polling_budget(WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT, true),
            WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT - super::BEST_EFFORT_DISCONNECT_TIMEOUT
        );
        assert_eq!(
            tunnel_polling_budget(WAIT_FOR_TUNNEL_STATE_TIMEOUT, false),
            WAIT_FOR_TUNNEL_STATE_TIMEOUT
        );
    }

    #[test]
    fn session_recreate_permit_is_one_shot() {
        assert!(super::may_attempt_session_recreate(false));
        assert!(!super::may_attempt_session_recreate(true));
    }

    #[tokio::test(start_paused = true)]
    async fn account_polling_recovers_after_transient_rpc_error() {
        let mut reader = FakeAccountStateReader::new([
            Err(rpc_failed()),
            Ok(ObservedAccountState::ReadyToConnect),
        ]);

        let state = poll_account_state_until(
            &mut reader,
            |state| matches!(state, ObservedAccountState::ReadyToConnect),
            Duration::from_secs(2),
        )
        .await
        .expect("account polling should recover");

        assert!(matches!(state, ObservedAccountState::ReadyToConnect));
    }

    #[tokio::test(start_paused = true)]
    async fn tunnel_polling_recovers_after_transient_rpc_error() {
        let mut reader = FakeTunnelStateReader::new([
            FakeRead::Result(Box::new(Err(rpc_failed()))),
            FakeRead::Result(Box::new(Ok(ObservedTunnelState::Disconnected))),
        ]);

        let state = poll_tunnel_state_until(
            &mut reader,
            |state| matches!(state, ObservedTunnelState::Disconnected),
            Duration::from_secs(2),
        )
        .await
        .expect("polling should recover");

        assert!(matches!(state, ObservedTunnelState::Disconnected));
    }

    #[tokio::test(start_paused = true)]
    async fn tunnel_polling_timeout_reports_last_observed_state() {
        let mut reader = FakeTunnelStateReader::new([FakeRead::Result(Box::new(Ok(
            ObservedTunnelState::Disconnected,
        )))]);
        let timeout = Duration::from_secs(2);
        let started = tokio::time::Instant::now();

        let error = poll_tunnel_state_until(&mut reader, |_| false, timeout)
            .await
            .expect_err("non-matching state must time out");

        assert_eq!(started.elapsed(), timeout);
        assert!(error.to_string().contains("last_observed=Disconnected"));
        assert!(error.to_string().contains("StillInState"));
    }

    #[tokio::test(start_paused = true)]
    async fn tunnel_polling_bounds_stalled_rpc() {
        let mut reader = FakeTunnelStateReader::new([FakeRead::Pending]);
        let timeout = Duration::from_secs(2);
        let started = tokio::time::Instant::now();

        let error = poll_tunnel_state_until(&mut reader, |_| false, timeout)
            .await
            .expect_err("stalled RPC must hit the polling deadline");

        assert_eq!(started.elapsed(), timeout);
        assert!(error.to_string().contains("RpcTimedOut"));
        assert!(error.to_string().contains("last_observed=<unavailable>"));
    }

    #[tokio::test(start_paused = true)]
    async fn failed_connect_polling_attempts_cleanup() {
        let mut reader = FakeTunnelStateReader::new([]);
        let mut disconnect = FakeDisconnectClient { disconnects: 0 };

        wait_for_tunnel_state_with_polling(
            &mut reader,
            &mut disconnect,
            |_| false,
            Duration::from_secs(2),
            true,
        )
        .await
        .expect_err("failed connect wait must return its polling error");

        assert_eq!(disconnect.disconnects, 1);
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
