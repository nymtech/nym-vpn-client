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
use std::{
    future::Future,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};
use test_rpc::{
    NymServiceClient,
    nym_daemon::{
        ObservedAccountState, ObservedAccountStateKind, ObservedTunnelState,
        ObservedTunnelStateKind, WaitOutcome,
    },
};

const BEST_EFFORT_DISCONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const DAEMON_QUIESCE_SETTLE: Duration = Duration::from_millis(250);
const HOST_OBSERVE_POLL_INTERVAL: Duration = Duration::from_millis(500);
const HOST_OBSERVE_RPC_TIMEOUT: Duration = Duration::from_secs(10);

trait TunnelObserver {
    async fn observe_tunnel(&self) -> Result<ObservedTunnelState, Error>;
}

trait AccountWaiter {
    async fn wait_account(
        &self,
        targets: Vec<ObservedAccountStateKind>,
        timeout: Duration,
    ) -> Result<WaitOutcome<ObservedAccountState>, Error>;
}

impl TunnelObserver for NymServiceClient {
    async fn observe_tunnel(&self) -> Result<ObservedTunnelState, Error> {
        Ok(self.get_observed_tunnel_state().await?)
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

    async fn recreate_after_disconnect_failure(&mut self) -> Result<bool, Error> {
        Ok(false)
    }
}

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
            self.rpc = Some(self.provider.recover_client_nym().await.map_err(|error| {
                NymClientError::Rpc(tonic::Status::unavailable(error.to_string()))
            })?);
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
        self.rpc = Some(
            self.provider
                .recover_client_nym()
                .await
                .map_err(Error::Other)?,
        );
        self.recreated = true;
        Ok(true)
    }
}

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
const MAX_THROTTLE_RETRIES: u32 = 5;

pub async fn login_idempotent(
    runner: &NymServiceClient,
    mut nym_client: NymProxyClient,
    provider: &RpcClientProvider,
) -> anyhow::Result<NymProxyClient> {
    match runner
        .get_observed_account_state()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get account state: {e}"))?
    {
        ObservedAccountState::ReadyToConnect => {
            return Ok(nym_client);
        }
        ObservedAccountState::LoggedOut => {
            nym_client = store_account_with_transport_retry(nym_client, provider).await?;
        }
        _ => {}
    }
    wait_for_account_state(runner, ObservedAccountState::ReadyToConnect)
        .await
        .map(drop)?;
    Ok(nym_client)
}

async fn store_account_with_transport_retry(
    mut nym_client: NymProxyClient,
    provider: &RpcClientProvider,
) -> anyhow::Result<NymProxyClient> {
    match store_account_idempotent(&mut nym_client).await {
        Ok(()) => Ok(nym_client),
        Err(error) if is_daemon_rpc_transport_error(&error) => {
            log::warn!(
                "store_account hit a dead DaemonRpc session; recreating client and retrying once: {error}"
            );
            drop(nym_client);
            let mut nym_client = provider.recover_client_nym().await?;
            store_account_idempotent(&mut nym_client).await?;
            Ok(nym_client)
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn is_daemon_rpc_transport_message(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("broken pipe")
        || message.contains("transport error")
        || message.contains("connection reset")
        || message.contains("h2 protocol error")
}

pub(crate) fn error_chain_has_daemon_rpc_transport(
    mut error: Option<&(dyn std::error::Error + 'static)>,
) -> bool {
    while let Some(err) = error {
        if is_daemon_rpc_transport_message(&err.to_string()) {
            return true;
        }
        error = err.source();
    }
    false
}

pub(crate) fn is_daemon_rpc_transport_error(error: &anyhow::Error) -> bool {
    error
        .chain()
        .any(|cause| is_daemon_rpc_transport_message(&cause.to_string()))
}

pub(crate) fn is_nym_client_transport_error(error: &NymClientError) -> bool {
    error_chain_has_daemon_rpc_transport(Some(error))
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisconnectRpcClass {
    Success,
    TransportRecoverable,
    Fatal,
}

#[cfg(test)]
fn classify_disconnect_rpc(result_ok: bool, error_message: Option<&str>) -> DisconnectRpcClass {
    if result_ok {
        DisconnectRpcClass::Success
    } else if error_message.is_some_and(is_daemon_rpc_transport_message) {
        DisconnectRpcClass::TransportRecoverable
    } else {
        DisconnectRpcClass::Fatal
    }
}

#[cfg(test)]
fn classify_disconnect_nym_error(result: &Result<bool, NymClientError>) -> DisconnectRpcClass {
    match result {
        Ok(_) => DisconnectRpcClass::Success,
        Err(err) if is_nym_client_transport_error(err) => DisconnectRpcClass::TransportRecoverable,
        Err(_) => DisconnectRpcClass::Fatal,
    }
}

fn throttle_backoff(attempts_so_far: u32) -> Option<Duration> {
    (attempts_so_far < MAX_THROTTLE_RETRIES).then_some(THROTTLE_RETRY_DELAY)
}

async fn store_account_idempotent(nym_client: &mut NymProxyClient) -> anyhow::Result<()> {
    let mut throttled_attempts = 0u32;
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
                throttled_attempts += 1;
                let Some(backoff) = throttle_backoff(throttled_attempts) else {
                    anyhow::bail!(
                        "store_account still throttled after {throttled_attempts} attempts: {}",
                        status.message()
                    );
                };
                log::debug!(
                    "Login throttled (attempt {throttled_attempts}/{MAX_THROTTLE_RETRIES}). Sleeping for {}s",
                    backoff.as_secs()
                );
                tokio::time::sleep(backoff).await;
            }
            Err(err) => {
                return Err(anyhow::Error::new(err).context("store_account RPC failed"));
            }
        }
    }
    Ok(())
}

pub async fn call_nym_with_transport_recovery<T, F, Fut>(
    provider: &RpcClientProvider,
    client: NymProxyClient,
    op: F,
) -> Result<(T, NymProxyClient), Error>
where
    F: Fn(NymProxyClient) -> Fut,
    Fut: Future<Output = (NymProxyClient, Result<T, NymClientError>)>,
{
    let (client, result) = op(client).await;
    match result {
        Ok(value) => Ok((value, client)),
        Err(err) if is_nym_client_transport_error(&err) => {
            log::warn!("DaemonRpc transport error on RPC; recovering once: {err:#}");
            drop(client);
            let client = provider.recover_client_nym().await.map_err(Error::Other)?;
            let (client, result) = op(client).await;
            match result {
                Ok(value) => Ok((value, client)),
                Err(err) => Err(Error::NymManagementInterface(err)),
            }
        }
        Err(err) => Err(Error::NymManagementInterface(err)),
    }
}

pub async fn ensure_daemon_rpc_responsive(
    provider: &RpcClientProvider,
    client: NymProxyClient,
) -> Result<NymProxyClient, Error> {
    let (_, client) = call_nym_with_transport_recovery(provider, client, |mut client| async move {
        let result = client.get_info().await.map(|_| ());
        (client, result)
    })
    .await?;
    Ok(client)
}

pub async fn replace_client_after_disconnect_prep(
    provider: &RpcClientProvider,
    client: NymProxyClient,
) -> Result<NymProxyClient, Error> {
    drop(client);
    provider.recover_client_nym().await.map_err(Error::Other)
}

pub async fn finish_prep_with_allow_lan(
    provider: &RpcClientProvider,
    client: NymProxyClient,
) -> Result<NymProxyClient, Error> {
    let allow_lan_result =
        call_nym_with_transport_recovery(provider, client, |mut client| async move {
            let result = client.set_allow_lan(true).await;
            (client, result)
        })
        .await;

    let prep_class = classify_allow_lan_prep(&allow_lan_result.as_ref().map(|_| ()));
    let client = match (prep_class, allow_lan_result) {
        (AllowLanPrepClass::ProbeOnly, Ok((_, client))) => client,
        (AllowLanPrepClass::RecoverThenProbe, Err(err)) => {
            log::warn!("Failed to enable allow_lan for diagnostics: {err:#}");
            provider.recover_client_nym().await.map_err(Error::Other)?
        }
        (AllowLanPrepClass::ProbeOnly, Err(_)) | (AllowLanPrepClass::RecoverThenProbe, Ok(_)) => {
            unreachable!("classify_allow_lan_prep disagrees with Result discriminant")
        }
    };

    ensure_daemon_rpc_responsive(provider, client).await
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AllowLanPrepClass {
    ProbeOnly,
    RecoverThenProbe,
}

fn classify_allow_lan_prep<T, E>(result: &Result<T, E>) -> AllowLanPrepClass {
    match result {
        Ok(_) => AllowLanPrepClass::ProbeOnly,
        Err(_) => AllowLanPrepClass::RecoverThenProbe,
    }
}

pub async fn set_enable_two_hop_with_recovery(
    provider: &RpcClientProvider,
    client: NymProxyClient,
    enable_two_hop: bool,
) -> Result<NymProxyClient, Error> {
    let (_, client) =
        call_nym_with_transport_recovery(provider, client, move |mut client| async move {
            let result = client.set_enable_two_hop(enable_two_hop).await;
            (client, result)
        })
        .await?;
    Ok(client)
}

pub async fn set_allow_lan_with_recovery(
    provider: &RpcClientProvider,
    client: NymProxyClient,
    allow_lan: bool,
) -> Result<NymProxyClient, Error> {
    let (_, client) =
        call_nym_with_transport_recovery(provider, client, move |mut client| async move {
            let result = client.set_allow_lan(allow_lan).await;
            (client, result)
        })
        .await?;
    Ok(client)
}

pub async fn connect_tunnel_with_recovery(
    provider: &RpcClientProvider,
    client: NymProxyClient,
) -> Result<NymProxyClient, Error> {
    let (_, client) = call_nym_with_transport_recovery(provider, client, |mut client| async move {
        let result = client.connect_tunnel().await;
        (client, result)
    })
    .await?;
    Ok(client)
}

pub async fn disconnect_and_wait(
    runner: &NymServiceClient,
    nym_client: NymProxyClient,
    provider: &RpcClientProvider,
) -> Result<NymProxyClient, Error> {
    log::trace!("Disconnecting");
    let nym_client = disconnect_with_transport_recovery(nym_client, provider).await?;

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

async fn disconnect_with_transport_recovery(
    mut nym_client: NymProxyClient,
    provider: &RpcClientProvider,
) -> Result<NymProxyClient, Error> {
    match nym_client.disconnect_tunnel().await {
        Ok(_) => Ok(nym_client),
        Err(err) if is_nym_client_transport_error(&err) => {
            log::warn!(
                "disconnect_tunnel hit a dead DaemonRpc session; recreating and retrying once: {err:#}"
            );
            drop(nym_client);
            nym_client = provider.recover_client_nym().await.map_err(Error::Other)?;
            match nym_client.disconnect_tunnel().await {
                Ok(_) => Ok(nym_client),
                Err(err2) if is_nym_client_transport_error(&err2) => {
                    log::warn!(
                        "disconnect_tunnel still broken after recreate; fresh client for Disconnected observe: {err2:#}"
                    );
                    drop(nym_client);
                    provider.recover_client_nym().await.map_err(Error::Other)
                }
                Err(err2) => Err(Error::NymManagementInterface(err2)),
            }
        }
        Err(err) => Err(Error::NymManagementInterface(err)),
    }
}

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

    let drop_daemon_rpc = matches!(expected, ExpectedTunnelState::Connected);
    if drop_daemon_rpc {
        log::debug!("quiescing serial DaemonRpc before Connected observe wait");
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
        let client = provider.recover_client_nym().await.map_err(Error::Other);
        return match merge_wait_and_client(observed, client) {
            Ok(pair) => Ok(pair),
            Err((error, client)) => {
                drop(client);
                Err(error)
            }
        };
    }

    let mut disconnect_client = ProviderDisconnectClient::new(provider);
    let observed = enforce_tunnel_wait_deadline(
        timeout,
        run_tunnel_wait(
            runner,
            &mut disconnect_client,
            vec![tunnel_target(&expected)],
            timeout,
            disconnect_on_timeout,
        ),
    )
    .await?;
    Ok((observed, nym_client))
}

pub(crate) async fn settle_daemon_rpc_quiesce() {
    tokio::time::sleep(DAEMON_QUIESCE_SETTLE).await;
}

pub(crate) fn merge_wait_and_client<S, C>(
    observed: Result<S, Error>,
    client: Result<C, Error>,
) -> Result<(S, C), (Error, Option<C>)> {
    match (observed, client) {
        (Ok(state), Ok(client)) => Ok((state, client)),
        (Err(wait_error), Ok(client)) => Err((wait_error, Some(client))),
        (Ok(_state), Err(recreate_error)) => Err((recreate_error, None)),
        (Err(wait_error), Err(_recreate_error)) => Err((wait_error, None)),
    }
}

pub(crate) fn tunnel_wait_params(expected: &ExpectedTunnelState) -> (Duration, bool) {
    match expected {
        ExpectedTunnelState::Connected => (WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT, true),
        ExpectedTunnelState::Disconnected => (WAIT_FOR_TUNNEL_STATE_TIMEOUT, false),
    }
}

pub(crate) fn tunnel_target(expected: &ExpectedTunnelState) -> ObservedTunnelStateKind {
    match expected {
        ExpectedTunnelState::Connected => ObservedTunnelStateKind::Connected,
        ExpectedTunnelState::Disconnected => ObservedTunnelStateKind::Disconnected,
    }
}

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

async fn run_tunnel_wait<O, D>(
    observer: &O,
    disconnect_client: &mut D,
    targets: Vec<ObservedTunnelStateKind>,
    timeout: Duration,
    disconnect_on_timeout: bool,
) -> Result<ObservedTunnelState, Error>
where
    O: TunnelObserver,
    D: DisconnectClient,
{
    let budget = tunnel_wait_budget(timeout, disconnect_on_timeout);
    let deadline = tokio::time::Instant::now() + budget;
    let mut last_observed = None;
    let mut last_error = None;

    log::debug!(
        "tunnel wait: host polling get_observed (budget={}s, disconnect_on_timeout={disconnect_on_timeout})",
        budget.as_secs()
    );

    let result = loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break tunnel_outcome_to_result(
                WaitOutcome::TimedOut {
                    last_observed: last_observed.clone(),
                },
                budget,
                last_error.clone(),
            );
        }

        let rpc_budget = HOST_OBSERVE_RPC_TIMEOUT.min(remaining);
        match tokio::time::timeout(rpc_budget, observer.observe_tunnel()).await {
            Ok(Ok(state)) => {
                if targets.iter().any(|target| target.matches(&state)) {
                    log::info!("tunnel wait: observed target {state:?}");
                    break Ok(state);
                }
                last_observed = Some(state);
            }
            Ok(Err(error)) => {
                log::warn!("tunnel wait: observe RPC failed (will retry): {error:?}");
                last_error = Some(format!("{error}"));
            }
            Err(_) => {
                log::debug!(
                    "tunnel wait: observe RPC timed out after {}ms (will retry)",
                    rpc_budget.as_millis()
                );
                last_error = Some(format!("observe RPC timed out after {rpc_budget:?}"));
            }
        }

        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break tunnel_outcome_to_result(
                WaitOutcome::TimedOut {
                    last_observed: last_observed.clone(),
                },
                budget,
                last_error.clone(),
            );
        }
        tokio::time::sleep(HOST_OBSERVE_POLL_INTERVAL.min(remaining)).await;
    };

    if let Err(error) = &result {
        log::error!("{error}");
        if disconnect_on_timeout {
            best_effort_disconnect(disconnect_client).await;
        }
    }

    result
}

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
    last_error: Option<String>,
) -> Result<ObservedTunnelState, Error> {
    match outcome {
        WaitOutcome::Reached(state) => Ok(state),
        WaitOutcome::TimedOut { last_observed } => Err(wait_timeout_error(
            "tunnel",
            budget,
            last_observed.as_ref().map(|state| format!("{state:?}")),
            last_error,
        )),
    }
}

fn wait_timeout_error(
    state_name: &str,
    budget: Duration,
    last_observed: Option<String>,
    last_error: Option<String>,
) -> Error {
    Error::Daemon(format!(
        "{state_name} state wait timed out after {}s; last_observed={}; last_error={}",
        budget.as_secs(),
        last_observed.unwrap_or_else(|| "<unavailable>".to_owned()),
        last_error.unwrap_or_else(|| "<none>".to_owned()),
    ))
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

async fn run_account_wait<W>(
    waiter: &W,
    targets: Vec<ObservedAccountStateKind>,
    timeout: Duration,
) -> Result<ObservedAccountState, Error>
where
    W: AccountWaiter,
{
    log::debug!(
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
                    None,
                )),
            }
        }
        Err(error) => {
            log::error!("account wait: guest RPC failed: {error}");
            Err(Error::Daemon(format!(
                "account state wait RPC failed: {error}"
            )))
        }
    }
}

pub const ROUNDTRIP_DNS_TIMEOUT: Duration = Duration::from_secs(30);

const DNS_RETRY_POLL_INTERVAL: Duration = Duration::from_millis(500);

pub async fn resolve_hostname_with_retry(
    rpc: &NymServiceClient,
    hostname: &str,
    timeout: Duration,
) -> anyhow::Result<Vec<SocketAddr>> {
    let owned = hostname.to_owned();
    let result = resolve_with_retry(hostname, timeout, DNS_RETRY_POLL_INTERVAL, || {
        let hostname = owned.clone();
        async move { rpc.resolve_hostname(hostname).await }
    })
    .await;

    if result.is_err() {
        log_dns_failure_diagnostics(rpc, hostname).await;
    }
    result
}

const DOT_UPSTREAM_PROBES: [SocketAddr; 2] = [
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(9, 9, 9, 9)), 853),
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 853),
];

const DNS_PROBE_BIND_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);

/// Best effort: a diagnostic that fails must not replace the resolution error.
async fn log_dns_failure_diagnostics(rpc: &NymServiceClient, hostname: &str) {
    for args in [["status", "tun1"], ["query", hostname]] {
        match rpc.exec("resolvectl", args).await {
            Ok(output) => log::error!(
                "resolvectl {}: {}{}",
                args.join(" "),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
            Err(error) => log::warn!("resolvectl {} failed in VM: {error}", args.join(" ")),
        }
    }

    let mut probes = Vec::with_capacity(DOT_UPSTREAM_PROBES.len());
    for dest in DOT_UPSTREAM_PROBES {
        let failure = rpc
            .send_tcp(None, DNS_PROBE_BIND_ADDR, dest)
            .await
            .err()
            .map(|error| error.to_string());
        probes.push((dest, failure));
    }
    log::error!("{}", summarize_upstream_probes(&probes));
}

fn summarize_upstream_probes(probes: &[(SocketAddr, Option<String>)]) -> String {
    let (unreachable, reachable): (Vec<_>, Vec<_>) =
        probes.iter().partition(|(_, failure)| failure.is_some());

    let reached = |entries: &[&(SocketAddr, Option<String>)]| {
        entries
            .iter()
            .map(|(addr, _)| addr.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };

    if unreachable.is_empty() {
        return format!(
            "tunnel reaches every DNS upstream ({}), so the daemon's local forwarder is the likely failure point",
            reached(&reachable)
        );
    }

    let reasons = unreachable
        .iter()
        .map(|(addr, failure)| {
            let reason = failure.as_deref().unwrap_or("unknown");
            format!("{addr}: {reason}")
        })
        .collect::<Vec<_>>()
        .join("; ");

    if reachable.is_empty() {
        format!("tunnel reaches no DNS upstream at all ({reasons})")
    } else {
        format!("tunnel reaches {} but not {reasons}", reached(&reachable))
    }
}

async fn resolve_with_retry<E, F, Fut>(
    hostname: &str,
    timeout: Duration,
    poll_interval: Duration,
    mut resolve: F,
) -> anyhow::Result<Vec<SocketAddr>>
where
    E: std::fmt::Display,
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<Vec<SocketAddr>, E>>,
{
    let mut last_outcome = "no attempt completed".to_string();

    let result = tokio::time::timeout(timeout, async {
        loop {
            match resolve().await {
                Ok(addrs) if !addrs.is_empty() => break addrs,
                Ok(_) => {
                    last_outcome = "resolver kept returning no addresses".to_string();
                    log::debug!("Got empty result for {hostname}, retrying...");
                }
                Err(error) => {
                    last_outcome = format!("last error: {error}");
                    log::debug!("DNS resolution of {hostname} failed: {error}, retrying...");
                }
            }
            tokio::time::sleep(poll_interval).await;
        }
    })
    .await;

    match result {
        Ok(addrs) => Ok(addrs),
        Err(_) => {
            let err = format!(
                "DNS resolution of {hostname} timed out after {}s ({last_outcome})",
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
        AccountWaiter, AllowLanPrepClass, DisconnectClient, DisconnectRpcClass,
        ExpectedTunnelState, TunnelObserver, account_target, classify_allow_lan_prep,
        classify_disconnect_nym_error, classify_disconnect_rpc, enforce_tunnel_wait_deadline,
        is_daemon_rpc_transport_error, is_daemon_rpc_transport_message,
        is_nym_client_transport_error, merge_wait_and_client, resolve_with_retry, run_account_wait,
        run_tunnel_wait, settle_daemon_rpc_quiesce, summarize_upstream_probes, tunnel_target,
        tunnel_wait_budget, tunnel_wait_params,
    };
    use crate::tests::{Error, WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT, WAIT_FOR_TUNNEL_STATE_TIMEOUT};
    use futures::StreamExt;
    use nym_vpn_proto::rpc_client::Error as NymClientError;
    use std::cell::Cell;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    fn sample_addr() -> std::net::SocketAddr {
        "93.184.216.34:443"
            .parse()
            .expect("literal is a socket addr")
    }

    /// The tunnel reports Connected before the exit gateway resolver answers, so the first
    /// queries after connect can fail or come back empty on a healthy tunnel.
    #[tokio::test(start_paused = true)]
    async fn resolve_retries_past_transport_errors_and_empty_answers() {
        let attempts = Cell::new(0);
        let addrs = resolve_with_retry(
            "nym.com",
            Duration::from_secs(30),
            Duration::from_millis(500),
            || {
                let attempt = attempts.get() + 1;
                attempts.set(attempt);
                async move {
                    match attempt {
                        1 => Err("resolver not ready"),
                        2 => Ok(Vec::new()),
                        _ => Ok(vec![sample_addr()]),
                    }
                }
            },
        )
        .await
        .expect("a resolver that eventually answers must succeed");

        assert_eq!(addrs, vec![sample_addr()]);
        assert_eq!(
            attempts.get(),
            3,
            "must stop querying once it has addresses"
        );
    }

    fn probe(addr: &str, failure: Option<&str>) -> (std::net::SocketAddr, Option<String>) {
        (
            addr.parse().expect("literal is a socket addr"),
            failure.map(str::to_string),
        )
    }

    #[test]
    fn upstream_probe_summary_separates_a_dead_tunnel_from_a_dead_forwarder() {
        let all_reachable =
            summarize_upstream_probes(&[probe("9.9.9.9:853", None), probe("1.1.1.1:853", None)]);
        assert!(all_reachable.contains("local forwarder"), "{all_reachable}");

        let none_reachable = summarize_upstream_probes(&[
            probe("9.9.9.9:853", Some("connection refused")),
            probe("1.1.1.1:853", Some("timed out")),
        ]);
        assert!(
            none_reachable.contains("no DNS upstream at all"),
            "{none_reachable}"
        );
        assert!(
            none_reachable.contains("connection refused"),
            "{none_reachable}"
        );
        assert!(none_reachable.contains("timed out"), "{none_reachable}");

        let mixed = summarize_upstream_probes(&[
            probe("9.9.9.9:853", None),
            probe("1.1.1.1:853", Some("timed out")),
        ]);
        assert!(mixed.contains("9.9.9.9:853"), "{mixed}");
        assert!(mixed.contains("1.1.1.1:853: timed out"), "{mixed}");
    }

    #[tokio::test(start_paused = true)]
    async fn resolve_gives_up_and_reports_why_when_answers_stay_empty() {
        let error = resolve_with_retry(
            "nym.com",
            Duration::from_secs(5),
            Duration::from_millis(500),
            || async { Ok::<_, String>(Vec::new()) },
        )
        .await
        .expect_err("an empty answer must never be reported as success");

        let rendered = error.to_string();
        assert!(rendered.contains("timed out after 5s"), "{rendered}");
        assert!(rendered.contains("no addresses"), "{rendered}");
    }

    #[tokio::test(start_paused = true)]
    async fn resolve_surfaces_the_last_transport_error_on_timeout() {
        let error = resolve_with_retry(
            "nym.com",
            Duration::from_secs(5),
            Duration::from_millis(500),
            || async { Err::<Vec<std::net::SocketAddr>, _>("connection refused") },
        )
        .await
        .expect_err("a resolver that never answers must fail");

        assert!(
            error.to_string().contains("connection refused"),
            "{error}, the CI log needs the reason, not just the timeout"
        );
    }
    use std::time::Duration;
    use test_rpc::nym_daemon::{
        ObservedAccountState, ObservedAccountStateKind, ObservedTunnelState,
        ObservedTunnelStateKind, ObservedTunnelType, WaitOutcome,
    };

    fn rpc_failed() -> Error {
        Error::NymManagementInterface(NymClientError::AuthenticationRequired)
    }

    fn connected_wg() -> ObservedTunnelState {
        ObservedTunnelState::Connected {
            tunnel_type: ObservedTunnelType::Wireguard,
        }
    }

    /// Scripted host-side observe polls for `run_tunnel_wait` unit tests.
    struct FakeTunnelObserver {
        polls: Mutex<VecDeque<Result<ObservedTunnelState, ()>>>,
    }

    impl FakeTunnelObserver {
        fn new(polls: Vec<Result<ObservedTunnelState, ()>>) -> Self {
            Self {
                polls: Mutex::new(polls.into()),
            }
        }
    }

    impl TunnelObserver for FakeTunnelObserver {
        async fn observe_tunnel(&self) -> Result<ObservedTunnelState, Error> {
            let next = self
                .polls
                .lock()
                .expect("poll script lock")
                .pop_front()
                .unwrap_or(Ok(ObservedTunnelState::Connecting));
            match next {
                Ok(state) => Ok(state),
                Err(()) => Err(rpc_failed()),
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

    #[tokio::test(start_paused = true)]
    async fn reached_tunnel_state_returns_without_cleanup() {
        let observer = FakeTunnelObserver::new(vec![
            Ok(ObservedTunnelState::Connecting),
            Ok(connected_wg()),
        ]);
        let mut disconnect = FakeDisconnectClient { disconnects: 0 };

        // Total deadline must exceed BEST_EFFORT_DISCONNECT_TIMEOUT so the poll budget is non-zero.
        let state = run_tunnel_wait(
            &observer,
            &mut disconnect,
            vec![ObservedTunnelStateKind::Connected],
            Duration::from_secs(15),
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

    #[tokio::test(start_paused = true)]
    async fn tunnel_timeout_reports_last_observed_and_cleans_up() {
        let observer = FakeTunnelObserver::new(vec![Ok(ObservedTunnelState::Connecting)]);
        let mut disconnect = FakeDisconnectClient { disconnects: 0 };

        let error = run_tunnel_wait(
            &observer,
            &mut disconnect,
            vec![ObservedTunnelStateKind::Connected],
            Duration::from_secs(11),
            true,
        )
        .await
        .expect_err("a timeout outcome must surface as an error");

        assert!(error.to_string().contains("tunnel state wait timed out"));
        assert!(error.to_string().contains("last_observed=Connecting"));
        assert_eq!(disconnect.disconnects, 1);
    }

    #[tokio::test(start_paused = true)]
    async fn tunnel_timeout_without_cleanup_skips_disconnect() {
        let observer = FakeTunnelObserver::new(vec![Ok(ObservedTunnelState::Connecting)]);
        let mut disconnect = FakeDisconnectClient { disconnects: 0 };

        run_tunnel_wait(
            &observer,
            &mut disconnect,
            vec![ObservedTunnelStateKind::Disconnected],
            Duration::from_millis(200),
            false,
        )
        .await
        .expect_err("timeout is an error");

        assert_eq!(disconnect.disconnects, 0);
    }

    #[tokio::test(start_paused = true)]
    async fn tunnel_observe_errors_retry_until_timeout_then_cleanup() {
        let observer = FakeTunnelObserver::new(vec![Err(()), Err(())]);
        let mut disconnect = FakeDisconnectClient { disconnects: 0 };

        let error = run_tunnel_wait(
            &observer,
            &mut disconnect,
            vec![ObservedTunnelStateKind::Connected],
            Duration::from_secs(11),
            true,
        )
        .await
        .expect_err("exhausted budget without a match must time out");

        assert!(error.to_string().contains("tunnel state wait timed out"));
        assert_eq!(disconnect.disconnects, 1);
    }

    #[test]
    fn throttle_retries_are_bounded() {
        assert_eq!(
            super::throttle_backoff(1),
            Some(super::THROTTLE_RETRY_DELAY)
        );
        assert_eq!(
            super::throttle_backoff(super::MAX_THROTTLE_RETRIES - 1),
            Some(super::THROTTLE_RETRY_DELAY)
        );
        assert_eq!(
            super::throttle_backoff(super::MAX_THROTTLE_RETRIES),
            None,
            "a permanently throttling account API must fail, not hang the run"
        );
    }

    /// Without this the real cause only reached the log stream, so a CI failure read as a
    /// bare timeout with no observed state.
    #[tokio::test(start_paused = true)]
    async fn tunnel_timeout_reports_the_last_observe_error() {
        let observer = FakeTunnelObserver::new(vec![Err(()), Err(())]);
        let mut disconnect = FakeDisconnectClient { disconnects: 0 };

        let error = run_tunnel_wait(
            &observer,
            &mut disconnect,
            vec![ObservedTunnelStateKind::Connected],
            Duration::from_secs(11),
            false,
        )
        .await
        .expect_err("exhausted budget without a match must time out");

        let rendered = error.to_string();
        assert!(
            rendered.contains(&rpc_failed().to_string()),
            "timeout must name the failure that kept retrying: {rendered}"
        );
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

    #[test]
    fn transport_error_classifier_matches_broken_daemon_rpc() {
        assert!(is_daemon_rpc_transport_error(&anyhow::anyhow!(
            "store_account RPC failed: transport error"
        )));
        assert!(is_daemon_rpc_transport_error(&anyhow::anyhow!(
            "broken pipe"
        )));
        assert!(is_daemon_rpc_transport_message(
            "Failed to disconnect: transport error: stream closed because of a broken pipe"
        ));
        assert!(!is_daemon_rpc_transport_error(&anyhow::anyhow!(
            "store_account error: ExistingAccount"
        )));
    }

    #[test]
    fn allow_lan_prep_classifies_ok_vs_recover_then_probe() {
        assert_eq!(
            classify_allow_lan_prep(&Ok::<(), &str>(())),
            AllowLanPrepClass::ProbeOnly
        );
        assert_eq!(
            classify_allow_lan_prep(&Err::<(), _>("Rpc call returned error")),
            AllowLanPrepClass::RecoverThenProbe
        );
        let transport = NymClientError::Rpc(tonic::Status::unknown(
            "transport error: connection error: broken pipe",
        ));
        assert!(is_nym_client_transport_error(&transport));
        assert_eq!(
            classify_allow_lan_prep(&Err::<(), _>(transport)),
            AllowLanPrepClass::RecoverThenProbe
        );
    }

    #[test]
    fn transport_classifier_walks_rpc_status_source_chain() {
        let err = NymClientError::Rpc(tonic::Status::unknown(
            "transport error: stream closed because of a broken pipe",
        ));
        assert_eq!(err.to_string(), "Rpc call returned error");
        assert!(!is_daemon_rpc_transport_message(&err.to_string()));
        assert!(is_nym_client_transport_error(&err));
        assert_eq!(
            classify_disconnect_nym_error(&Err(err)),
            DisconnectRpcClass::TransportRecoverable
        );

        let allow_lan_shaped = NymClientError::Rpc(tonic::Status::unknown(
            "transport error: connection error: broken pipe",
        ));
        assert!(is_nym_client_transport_error(&allow_lan_shaped));

        let chained = anyhow::Error::new(NymClientError::Rpc(tonic::Status::unknown(
            "transport error: stream closed because of a broken pipe",
        )))
        .context("Failed to disconnect");
        assert!(is_daemon_rpc_transport_error(&chained));

        assert!(!is_nym_client_transport_error(
            &NymClientError::AuthenticationRequired
        ));
        assert!(!is_daemon_rpc_transport_error(&anyhow::anyhow!(
            "Rpc call returned error"
        )));

        let store_shaped = anyhow::Error::new(NymClientError::Rpc(tonic::Status::unknown(
            "transport error: stream closed because of a broken pipe",
        )))
        .context("store_account RPC failed");
        assert!(is_daemon_rpc_transport_error(&store_shaped));
        let lost_source = anyhow::anyhow!("store_account RPC failed: Rpc call returned error");
        assert!(!is_daemon_rpc_transport_error(&lost_source));
    }

    #[test]
    fn disconnect_rpc_classifier_selects_recovery_branches() {
        assert_eq!(
            classify_disconnect_rpc(true, None),
            DisconnectRpcClass::Success
        );
        assert_eq!(
            classify_disconnect_rpc(
                false,
                Some("transport error: stream closed because of a broken pipe")
            ),
            DisconnectRpcClass::TransportRecoverable
        );
        assert_eq!(
            classify_disconnect_rpc(false, Some("authentication required")),
            DisconnectRpcClass::Fatal
        );
        assert_eq!(
            classify_disconnect_rpc(false, None),
            DisconnectRpcClass::Fatal
        );
        assert_eq!(
            classify_disconnect_nym_error(&Ok(true)),
            DisconnectRpcClass::Success
        );
    }

    #[test]
    fn connected_wait_drops_daemon_rpc_but_disconnect_does_not() {
        assert!(matches!(
            tunnel_wait_params(&ExpectedTunnelState::Connected),
            (WAIT_FOR_TUNNEL_CONNECTED_TIMEOUT, true)
        ));
        assert!(matches!(
            tunnel_wait_params(&ExpectedTunnelState::Disconnected),
            (WAIT_FOR_TUNNEL_STATE_TIMEOUT, false)
        ));
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
    fn merge_wait_returns_client_on_observe_error() {
        let observed: Result<ObservedTunnelState, Error> =
            Err(Error::Daemon("observe failed".into()));
        let client: Result<&str, Error> = Ok("recreated");

        let err =
            merge_wait_and_client(observed, client).expect_err("observe failure must surface");
        assert!(err.0.to_string().contains("observe failed"));
        assert_eq!(err.1, Some("recreated"));
    }

    #[test]
    fn merge_wait_success_pairs_state_and_client() {
        let observed = Ok(ObservedTunnelState::Disconnected);
        let client: Result<&str, Error> = Ok("recreated");

        let (state, got) = merge_wait_and_client(observed, client).expect("success pairs both");
        assert!(matches!(state, ObservedTunnelState::Disconnected));
        assert_eq!(got, "recreated");
    }

    #[test]
    fn merge_wait_prefers_observe_error_when_recreate_also_fails() {
        let observed: Result<ObservedTunnelState, Error> =
            Err(Error::Daemon("observe failed".into()));
        let client: Result<&str, Error> = Err(Error::Daemon("recreate failed".into()));

        let err =
            merge_wait_and_client(observed, client).expect_err("combined failure must surface");
        assert!(err.0.to_string().contains("observe failed"));
        assert!(err.1.is_none());
    }

    #[tokio::test]
    async fn provider_dangling_recover_requests_a_new_duplex() {
        let (provider, mut rx) = crate::nym_daemon::RpcClientProvider::dangling_for_tests();
        let create = tokio::spawn(async move { provider.recover_client_nym().await });
        let _duplex = tokio::time::timeout(Duration::from_secs(2), rx.next())
            .await
            .expect("recover_client_nym must request a duplex after settle")
            .expect("channel closed");
        create.abort();
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
