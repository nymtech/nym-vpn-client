// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{
    Error, WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT, WAIT_FOR_TUNNEL_STATE_TIMEOUT,
    config_nym::TEST_CONFIG_NYM,
};
use crate::nym_daemon::RpcClientProvider;
use nym_vpn_lib_types::{AccountCommandError, AccountControllerState, TunnelState};
use nym_vpn_proto::rpc_client::{Error as NymClientError, RpcClient as NymProxyClient};
use std::{fmt::Debug, future::Future, net::SocketAddr, time::Duration};
use test_rpc::NymServiceClient;

/// Bounded best-effort disconnect after a tunnel wait timeout. Must not nest a full
/// `disconnect_and_wait` (that would block the suite for another 40s on a dead serial).
const BEST_EFFORT_DISCONNECT_TIMEOUT: Duration = Duration::from_secs(10);

const TUNNEL_STATE_POLL_DELAY: Duration = Duration::from_millis(500);
const TUNNEL_STATE_RPC_TIMEOUT: Duration = Duration::from_secs(30);

trait StateClient<S> {
    async fn read_state(&mut self) -> Result<S, NymClientError>;

    /// Drop and recreate the underlying RPC session once after a stall. Returns `true`
    /// when a new session was established and the caller should retry immediately.
    async fn recreate_after_rpc_stall(&mut self) -> Result<bool, Error> {
        Ok(false)
    }
}

trait TunnelStateClient: StateClient<TunnelState> {
    async fn disconnect_tunnel(&mut self) -> Result<bool, NymClientError>;
}

trait AccountStateClient: StateClient<AccountControllerState> {}

impl<T> AccountStateClient for T where T: StateClient<AccountControllerState> {}

/// Replaces `NymProxyClient` once via `RpcClientProvider` when serial/HTTP2 appears poisoned.
struct RecreatingNymClient<'a> {
    rpc: &'a mut NymProxyClient,
    provider: &'a RpcClientProvider,
    recreated: bool,
}

impl RecreatingNymClient<'_> {
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

impl StateClient<TunnelState> for RecreatingNymClient<'_> {
    async fn read_state(&mut self) -> Result<TunnelState, NymClientError> {
        self.rpc.get_tunnel_state().await
    }

    async fn recreate_after_rpc_stall(&mut self) -> Result<bool, Error> {
        self.recreate_once().await
    }
}

impl TunnelStateClient for RecreatingNymClient<'_> {
    async fn disconnect_tunnel(&mut self) -> Result<bool, NymClientError> {
        NymProxyClient::disconnect_tunnel(self.rpc).await
    }
}

impl StateClient<AccountControllerState> for RecreatingNymClient<'_> {
    async fn read_state(&mut self) -> Result<AccountControllerState, NymClientError> {
        self.rpc.get_account_state().await
    }

    async fn recreate_after_rpc_stall(&mut self) -> Result<bool, Error> {
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

pub async fn login_idempotent(
    nym_client: &mut NymProxyClient,
    provider: &RpcClientProvider,
) -> anyhow::Result<()> {
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
    wait_for_account_state(nym_client, provider, AccountControllerState::ReadyToConnect)
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
    nym_client: &mut NymProxyClient,
    provider: &RpcClientProvider,
) -> Result<(), Error> {
    log::trace!("Disconnecting");
    nym_client.disconnect_tunnel().await?;

    wait_for_tunnel_state_fn(
        nym_client,
        provider,
        |state| matches!(state, TunnelState::Disconnected),
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
    C: TunnelStateClient,
{
    match tokio::time::timeout(BEST_EFFORT_DISCONNECT_TIMEOUT, client.disconnect_tunnel()).await {
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
    provider: &RpcClientProvider,
    expected: ExpectedTunnelState,
) -> Result<TunnelState, Error> {
    let (timeout, disconnect_on_timeout) = tunnel_wait_params(&expected);
    log::debug!(
        "Waiting for tunnel state: {expected:?} (timeout: {}s)",
        timeout.as_secs()
    );
    wait_for_tunnel_state_fn(
        rpc,
        provider,
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

/// Wait for the tunnel to reach a persistent state using bounded unary RPCs. The serial transport
/// does not reliably deliver the long-lived event stream used by local socket clients.
pub async fn wait_for_tunnel_state_fn(
    rpc: &mut NymProxyClient,
    provider: &RpcClientProvider,
    accept_state_fn: impl Fn(&TunnelState) -> bool,
    timeout: Duration,
    disconnect_on_timeout: bool,
) -> Result<TunnelState, Error> {
    let mut client = RecreatingNymClient {
        rpc,
        provider,
        recreated: false,
    };
    enforce_tunnel_wait_deadline(
        timeout,
        wait_for_tunnel_state_with_polling(
            &mut client,
            accept_state_fn,
            timeout,
            disconnect_on_timeout,
        ),
    )
    .await
}

async fn wait_for_tunnel_state_with_polling<C>(
    rpc: &mut C,
    accept_state_fn: impl Fn(&TunnelState) -> bool,
    timeout: Duration,
    disconnect_on_timeout: bool,
) -> Result<TunnelState, Error>
where
    C: TunnelStateClient,
{
    let polling_budget = tunnel_polling_budget(timeout, disconnect_on_timeout);

    match poll_tunnel_state_until(rpc, accept_state_fn, polling_budget).await {
        Ok(state) => Ok(state),
        Err(error) => {
            log::error!("{error}");
            if disconnect_on_timeout {
                best_effort_disconnect(rpc).await;
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
    accept_state_fn: impl Fn(&TunnelState) -> bool,
    timeout: Duration,
) -> Result<TunnelState, Error>
where
    R: TunnelStateClient,
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
                // Do not recreate on RpcFailed: a quick application error must not tear
                // down a healthy serial session. Recreate only on client-side timeout.
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
                if reader.recreate_after_rpc_stall().await? {
                    log::warn!("poll[{state_name}#{attempt}] recreated RPC session after timeout");
                    continue;
                }
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
    rpc: &mut NymProxyClient,
    provider: &RpcClientProvider,
    expected: AccountControllerState,
) -> Result<AccountControllerState, Error> {
    let timeout = Duration::from_secs(60);
    log::debug!(
        "Waiting for account state: {expected:?} (timeout: {}s)",
        timeout.as_secs()
    );
    wait_for_account_state_fn(rpc, provider, move |state| state.eq(&expected), timeout).await
}

/// Wait for account readiness without opening a long-lived stream over the serial transport.
pub async fn wait_for_account_state_fn(
    rpc: &mut NymProxyClient,
    provider: &RpcClientProvider,
    accept_state_fn: impl Fn(&AccountControllerState) -> bool,
    timeout: Duration,
) -> Result<AccountControllerState, Error> {
    let mut client = RecreatingNymClient {
        rpc,
        provider,
        recreated: false,
    };
    poll_account_state_until(&mut client, accept_state_fn, timeout).await
}

async fn poll_account_state_until<R>(
    reader: &mut R,
    accept_state_fn: impl Fn(&AccountControllerState) -> bool,
    timeout: Duration,
) -> Result<AccountControllerState, Error>
where
    R: AccountStateClient,
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
        ExpectedTunnelState, StateClient, TunnelStateClient, enforce_tunnel_wait_deadline,
        poll_account_state_until, poll_tunnel_state_until, tunnel_polling_budget,
        tunnel_wait_params, wait_for_tunnel_state_with_polling,
    };
    use crate::tests::{WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT, WAIT_FOR_TUNNEL_STATE_TIMEOUT};
    use futures::StreamExt;
    use nym_vpn_lib_types::{AccountControllerState, TunnelState};
    use nym_vpn_proto::rpc_client::Error as NymClientError;
    use std::{collections::VecDeque, future::pending, time::Duration};

    enum FakeRead {
        Result(Box<Result<TunnelState, NymClientError>>),
        Pending,
    }

    struct FakeTunnelStateReader {
        reads: VecDeque<FakeRead>,
        disconnects: usize,
        recreates: usize,
        reads_after_recreate: Option<VecDeque<FakeRead>>,
        pending_after_recreate: bool,
    }

    impl FakeTunnelStateReader {
        fn new(reads: impl IntoIterator<Item = FakeRead>) -> Self {
            Self {
                reads: reads.into_iter().collect(),
                disconnects: 0,
                recreates: 0,
                reads_after_recreate: None,
                pending_after_recreate: false,
            }
        }

        fn with_reads_after_recreate(mut self, reads: impl IntoIterator<Item = FakeRead>) -> Self {
            self.reads_after_recreate = Some(reads.into_iter().collect());
            self
        }

        fn pending_after_recreate(mut self) -> Self {
            self.pending_after_recreate = true;
            self
        }
    }

    struct FakeAccountStateReader {
        reads: VecDeque<Result<AccountControllerState, NymClientError>>,
        recreates: usize,
        reads_after_recreate: Option<VecDeque<Result<AccountControllerState, NymClientError>>>,
    }

    impl FakeAccountStateReader {
        fn new(
            reads: impl IntoIterator<Item = Result<AccountControllerState, NymClientError>>,
        ) -> Self {
            Self {
                reads: reads.into_iter().collect(),
                recreates: 0,
                reads_after_recreate: None,
            }
        }

        fn with_reads_after_recreate(
            mut self,
            reads: impl IntoIterator<Item = Result<AccountControllerState, NymClientError>>,
        ) -> Self {
            self.reads_after_recreate = Some(reads.into_iter().collect());
            self
        }
    }

    impl StateClient<AccountControllerState> for FakeAccountStateReader {
        async fn read_state(&mut self) -> Result<AccountControllerState, NymClientError> {
            self.reads
                .pop_front()
                .unwrap_or(Ok(AccountControllerState::ReadyToConnect))
        }

        async fn recreate_after_rpc_stall(&mut self) -> Result<bool, super::Error> {
            if self.recreates > 0 {
                return Ok(false);
            }
            self.recreates += 1;
            if let Some(reads) = self.reads_after_recreate.take() {
                self.reads = reads;
            }
            Ok(true)
        }
    }

    impl StateClient<TunnelState> for FakeTunnelStateReader {
        async fn read_state(&mut self) -> Result<TunnelState, NymClientError> {
            let next = self.reads.pop_front().or_else(|| {
                if self.pending_after_recreate && self.recreates > 0 {
                    Some(FakeRead::Pending)
                } else {
                    None
                }
            });
            match next.unwrap_or(FakeRead::Result(Box::new(Ok(TunnelState::Disconnected)))) {
                FakeRead::Result(result) => *result,
                FakeRead::Pending => pending().await,
            }
        }

        async fn recreate_after_rpc_stall(&mut self) -> Result<bool, super::Error> {
            if self.recreates > 0 {
                return Ok(false);
            }
            self.recreates += 1;
            if let Some(reads) = self.reads_after_recreate.take() {
                self.reads = reads;
            }
            Ok(true)
        }
    }

    impl TunnelStateClient for FakeTunnelStateReader {
        async fn disconnect_tunnel(&mut self) -> Result<bool, NymClientError> {
            self.disconnects += 1;
            Ok(true)
        }
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

    #[tokio::test(start_paused = true)]
    async fn account_polling_recovers_after_transient_rpc_error() {
        let mut reader = FakeAccountStateReader::new([
            Err(NymClientError::AuthenticationRequired),
            Ok(AccountControllerState::ReadyToConnect),
        ]);

        let state = poll_account_state_until(
            &mut reader,
            |state| matches!(state, AccountControllerState::ReadyToConnect),
            Duration::from_secs(2),
        )
        .await
        .expect("account polling should recover");

        assert!(matches!(state, AccountControllerState::ReadyToConnect));
        assert_eq!(
            reader.recreates, 0,
            "RpcFailed must not recreate the serial session"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn account_polling_recreates_session_after_stall_and_recovers() {
        struct HangThenAccount {
            inner: FakeAccountStateReader,
            first: bool,
        }
        impl StateClient<AccountControllerState> for HangThenAccount {
            async fn read_state(&mut self) -> Result<AccountControllerState, NymClientError> {
                if self.first {
                    self.first = false;
                    pending().await
                } else {
                    self.inner.read_state().await
                }
            }
            async fn recreate_after_rpc_stall(&mut self) -> Result<bool, super::Error> {
                self.inner.recreate_after_rpc_stall().await
            }
        }
        let mut reader = HangThenAccount {
            inner: FakeAccountStateReader::new([])
                .with_reads_after_recreate([Ok(AccountControllerState::ReadyToConnect)]),
            first: true,
        };

        let state = poll_account_state_until(
            &mut reader,
            |state| matches!(state, AccountControllerState::ReadyToConnect),
            Duration::from_secs(60),
        )
        .await
        .expect("account recreate after stall should recover");

        assert!(matches!(state, AccountControllerState::ReadyToConnect));
        assert_eq!(reader.inner.recreates, 1);
    }

    #[test]
    fn session_recreate_permit_is_one_shot() {
        assert!(super::may_attempt_session_recreate(false));
        assert!(!super::may_attempt_session_recreate(true));
    }

    #[tokio::test(start_paused = true)]
    async fn tunnel_polling_rpc_failed_does_not_recreate_session() {
        let mut reader = FakeTunnelStateReader::new([
            FakeRead::Result(Box::new(Err(NymClientError::AuthenticationRequired))),
            FakeRead::Result(Box::new(Ok(TunnelState::Disconnected))),
        ]);

        let state = poll_tunnel_state_until(
            &mut reader,
            |state| matches!(state, TunnelState::Disconnected),
            Duration::from_secs(2),
        )
        .await
        .expect("RpcFailed should soft-recover without recreate");

        assert!(matches!(state, TunnelState::Disconnected));
        assert_eq!(reader.recreates, 0);
    }

    #[tokio::test(start_paused = true)]
    async fn provider_dangling_create_is_requested_on_new_client_nym() {
        let (provider, mut rx) = crate::nym_daemon::RpcClientProvider::dangling_for_tests();
        let create = tokio::spawn(async move { provider.new_client_nym().await });
        let channel = tokio::time::timeout(Duration::from_secs(1), rx.next())
            .await
            .expect("recreate path must request a management duplex")
            .expect("channel remains open");
        drop(channel);
        create.abort();
    }

    #[tokio::test(start_paused = true)]
    async fn tunnel_polling_recovers_after_transient_rpc_error() {
        let mut reader = FakeTunnelStateReader::new([
            FakeRead::Result(Box::new(Err(NymClientError::AuthenticationRequired))),
            FakeRead::Result(Box::new(Ok(TunnelState::Disconnected))),
        ]);

        let state = poll_tunnel_state_until(
            &mut reader,
            |state| matches!(state, TunnelState::Disconnected),
            Duration::from_secs(2),
        )
        .await
        .expect("polling should recover");

        assert!(matches!(state, TunnelState::Disconnected));
    }

    #[tokio::test(start_paused = true)]
    async fn tunnel_polling_timeout_reports_last_observed_state() {
        let mut reader =
            FakeTunnelStateReader::new([FakeRead::Result(Box::new(Ok(TunnelState::Disconnected)))]);
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
        // Disable one-shot recreate so a permanent stall still hits the deadline.
        reader.recreates = 1;
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
    async fn tunnel_polling_recreates_session_after_stall_and_recovers() {
        let mut reader = FakeTunnelStateReader::new([FakeRead::Pending])
            .with_reads_after_recreate([FakeRead::Result(Box::new(Ok(TunnelState::Disconnected)))]);

        // Stall consumes TUNNEL_STATE_RPC_TIMEOUT (30s); leave budget for the post-recreate poll.
        let state = poll_tunnel_state_until(
            &mut reader,
            |state| matches!(state, TunnelState::Disconnected),
            Duration::from_secs(60),
        )
        .await
        .expect("recreate after stall should recover");

        assert!(matches!(state, TunnelState::Disconnected));
        assert_eq!(reader.recreates, 1);
    }

    #[tokio::test(start_paused = true)]
    async fn tunnel_polling_recreates_session_at_most_once() {
        let mut reader = FakeTunnelStateReader::new([FakeRead::Pending]).pending_after_recreate();
        let timeout = Duration::from_secs(90);
        let started = tokio::time::Instant::now();

        let error = poll_tunnel_state_until(&mut reader, |_| false, timeout)
            .await
            .expect_err("second stall after recreate must time out");

        assert_eq!(started.elapsed(), timeout);
        assert_eq!(reader.recreates, 1);
        assert!(error.to_string().contains("RpcTimedOut"));
    }

    #[tokio::test(start_paused = true)]
    async fn failed_connect_polling_attempts_cleanup() {
        let mut reader = FakeTunnelStateReader::new([]);
        reader.recreates = 1;

        wait_for_tunnel_state_with_polling(&mut reader, |_| false, Duration::from_secs(2), true)
            .await
            .expect_err("failed connect wait must return its polling error");

        assert_eq!(reader.disconnects, 1);
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
