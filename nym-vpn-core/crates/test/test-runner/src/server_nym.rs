// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app_nymvpn, forward, get_nymvpn_pipe_status, logging, logging::LOGGER, net, package_nym, sys,
    util, util::OnDrop,
};
use futures::{FutureExt, select, select_biased};
use nym_vpn_lib_types::{AccountControllerState, TunnelState, TunnelType};
use nym_vpn_proto::rpc_client::RpcClient as NymDaemonClient;
use std::{
    collections::{BTreeMap, HashMap},
    future::Future,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    process::Stdio,
    sync::Arc,
    time::Duration,
};
use tarpc::context;
use test_rpc::{
    AppTrace, Service, SpawnOpts, UNPRIVILEGED_USER,
    meta::OsVersion,
    net::SockHandleId,
    nym_daemon::{
        ObservedAccountState, ObservedAccountStateKind, ObservedTunnelState,
        ObservedTunnelStateKind, ObservedTunnelType, WaitOutcome,
    },
    package::Package,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{ChildStdin, ChildStdout, Command},
    sync::{Mutex, broadcast::error::TryRecvError, oneshot},
    task,
    time::{Instant, sleep},
};

/// Cadence of the guest-local daemon poll inside a blocking state wait. Small enough to
/// return promptly after a transition, large enough to keep local UDS load negligible.
const OBSERVE_POLL_INTERVAL: Duration = Duration::from_millis(250);
/// Upper bound on a single local UDS state read so a hung daemon call cannot stall past
/// the wait deadline (mirrors the former host-side per-poll RPC timeout).
const OBSERVE_READ_TIMEOUT: Duration = Duration::from_secs(30);

#[cfg(target_os = "linux")]
const NYM_SYSTEMD_SERVICE_NAME: &str = "nymvpnd.service";

#[derive(Clone, Default)]
pub struct NymTestServer(Arc<Mutex<NymServerState>>);

#[derive(Default)]
struct NymServerState {
    spawned_procs: HashMap<u32, NymSpawnedProcess>,
}

struct NymSpawnedProcess {
    stdout: Option<ChildStdout>,
    stdin: Option<ChildStdin>,

    #[allow(dead_code)]
    abort_handle: OnDrop,
}

#[tarpc::server]
impl Service for NymTestServer {
    async fn install_app(
        self,
        _: context::Context,
        package: Package,
    ) -> Result<(), test_rpc::Error> {
        log::debug!("Installing app");

        package_nym::install_package(package).await?;

        log::debug!("Install complete");

        Ok(())
    }

    async fn uninstall_app(
        self,
        _: context::Context,
        env: HashMap<String, String>,
    ) -> Result<(), test_rpc::Error> {
        log::debug!("Uninstalling app");

        package_nym::uninstall_app(env).await?;

        log::debug!("Uninstalled app");

        Ok(())
    }

    async fn exec(
        self,
        _: context::Context,
        path: String,
        args: Vec<String>,
        env: BTreeMap<String, String>,
    ) -> Result<test_rpc::ExecResult, test_rpc::Error> {
        log::debug!("Exec {} (args: {args:?})", path);

        let mut cmd = Command::new(&path);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::piped());
        cmd.args(args);

        #[cfg(target_os = "windows")]
        {
            // Make sure that PATH is updated
            cmd.env("PATH", sys::get_system_path_var()?);
            if let Some(home_dir) = dirs::home_dir() {
                cmd.env("USERPROFILE", home_dir);
            }
        }

        #[cfg(unix)]
        if let Some(home_dir) = dirs::home_dir() {
            cmd.env("HOME", home_dir);
        }

        cmd.envs(env);

        let output = cmd.output().await.map_err(|error| {
            log::error!("Failed to exec {}: {error}", path);
            test_rpc::Error::Syscall
        })?;

        let result = test_rpc::ExecResult {
            code: output.status.code(),
            stdout: output.stdout,
            stderr: output.stderr,
        };

        log::debug!("Finished exec: {:?}", result.code);

        Ok(result)
    }

    async fn nymvpn_daemon_get_status(
        self,
        _: context::Context,
    ) -> test_rpc::nym_daemon::ServiceStatus {
        get_nymvpn_pipe_status()
    }

    async fn get_observed_tunnel_state(
        self,
        _: context::Context,
    ) -> Result<ObservedTunnelState, test_rpc::Error> {
        let mut client = NymDaemonClient::new()
            .await
            .map_err(|error| test_rpc::Error::DaemonRpc(error.to_string()))?;
        let state = client
            .get_tunnel_state()
            .await
            .map_err(|error| test_rpc::Error::DaemonRpc(error.to_string()))?;
        Ok(observed_tunnel_state(state))
    }

    async fn get_observed_account_state(
        self,
        _: context::Context,
    ) -> Result<ObservedAccountState, test_rpc::Error> {
        let mut client = NymDaemonClient::new()
            .await
            .map_err(|error| test_rpc::Error::DaemonRpc(error.to_string()))?;
        let state = client
            .get_account_state()
            .await
            .map_err(|error| test_rpc::Error::DaemonRpc(error.to_string()))?;
        Ok(observed_account_state(state))
    }

    async fn wait_for_observed_tunnel_state(
        self,
        _: context::Context,
        targets: Vec<ObservedTunnelStateKind>,
        timeout_ms: u64,
    ) -> Result<WaitOutcome<ObservedTunnelState>, test_rpc::Error> {
        if targets.is_empty() {
            return Err(test_rpc::Error::Other(
                "wait_for_observed_tunnel_state requires at least one target".into(),
            ));
        }
        let client = NymDaemonClient::new()
            .await
            .map_err(|error| test_rpc::Error::DaemonRpc(error.to_string()))?;
        wait_for_observed(
            timeout_ms,
            || {
                let mut client = client.clone();
                async move {
                    client
                        .get_tunnel_state()
                        .await
                        .map(observed_tunnel_state)
                        .map_err(|error| test_rpc::Error::DaemonRpc(error.to_string()))
                }
            },
            |state| targets.iter().any(|target| target.matches(state)),
        )
        .await
    }

    async fn wait_for_observed_account_state(
        self,
        _: context::Context,
        targets: Vec<ObservedAccountStateKind>,
        timeout_ms: u64,
    ) -> Result<WaitOutcome<ObservedAccountState>, test_rpc::Error> {
        if targets.is_empty() {
            return Err(test_rpc::Error::Other(
                "wait_for_observed_account_state requires at least one target".into(),
            ));
        }
        let client = NymDaemonClient::new()
            .await
            .map_err(|error| test_rpc::Error::DaemonRpc(error.to_string()))?;
        wait_for_observed(
            timeout_ms,
            || {
                let mut client = client.clone();
                async move {
                    client
                        .get_account_state()
                        .await
                        .map(observed_account_state)
                        .map_err(|error| test_rpc::Error::DaemonRpc(error.to_string()))
                }
            },
            |state| targets.iter().any(|target| target.matches(state)),
        )
        .await
    }

    /// Get the installed app version
    async fn nymvpn_version(self, _: context::Context) -> Result<String, test_rpc::Error> {
        app_nymvpn::version().await
    }

    /// refers to Nym daemon in this case
    async fn find_nymvpn_app_traces(
        self,
        _: context::Context,
    ) -> Result<Vec<AppTrace>, test_rpc::Error> {
        app_nymvpn::find_traces()
    }

    async fn get_nym_app_cache_dir(self, _: context::Context) -> Result<PathBuf, test_rpc::Error> {
        app_nymvpn::find_cache_traces()
    }

    async fn send_tcp(
        self,
        _: context::Context,
        interface: Option<String>,
        bind_addr: SocketAddr,
        destination: SocketAddr,
    ) -> Result<(), test_rpc::Error> {
        net::send_tcp(interface, bind_addr, destination).await
    }

    async fn send_udp(
        self,
        _: context::Context,
        interface: Option<String>,
        bind_addr: SocketAddr,
        destination: SocketAddr,
    ) -> Result<(), test_rpc::Error> {
        net::send_udp(interface, bind_addr, destination).await
    }

    async fn send_ping(
        self,
        _: context::Context,
        destination: IpAddr,
        interface: Option<String>,
        size: usize,
    ) -> Result<(), test_rpc::Error> {
        net::send_ping(destination, interface.as_deref(), size)
            .await
            .map_err(|e| test_rpc::Error::Ping(e.to_string()))
    }

    async fn geoip_lookup(
        self,
        _ctx: context::Context,
    ) -> Result<test_rpc::AmIMullvad, test_rpc::Error> {
        unimplemented!()
    }

    async fn resolve_hostname(
        self,
        _: context::Context,
        hostname: String,
    ) -> Result<Vec<SocketAddr>, test_rpc::Error> {
        Ok(tokio::net::lookup_host(&format!("{hostname}:0"))
            .await
            .map_err(|error| {
                log::debug!("resolve_hostname failed: {error}");
                test_rpc::Error::DnsResolution
            })?
            .collect())
    }

    async fn start_tcp_forward(
        self,
        _: context::Context,
        bind_addr: SocketAddr,
        via_addr: SocketAddr,
    ) -> Result<(SockHandleId, SocketAddr), test_rpc::Error> {
        forward::start_server(bind_addr, via_addr).await
    }

    async fn stop_tcp_forward(
        self,
        _: context::Context,
        id: SockHandleId,
    ) -> Result<(), test_rpc::Error> {
        forward::stop_server(id)
    }

    async fn get_interface_ip(
        self,
        _: context::Context,
        interface: String,
    ) -> Result<IpAddr, test_rpc::Error> {
        net::get_interface_ip(&interface)
    }

    async fn get_interface_mtu(
        self,
        _: context::Context,
        interface: String,
    ) -> Result<u16, test_rpc::Error> {
        net::get_interface_mtu(&interface)
    }

    async fn get_interface_mac(
        self,
        _: context::Context,
        interface: String,
    ) -> Result<Option<[u8; 6]>, test_rpc::Error> {
        net::get_interface_mac(&interface)
    }

    async fn get_default_interface(self, _: context::Context) -> Result<String, test_rpc::Error> {
        Ok(net::get_default_interface().to_owned())
    }

    async fn poll_output(
        self,
        _: context::Context,
    ) -> Result<Vec<test_rpc::logging::Output>, test_rpc::Error> {
        let mut listener = LOGGER.0.lock().await;
        if let Ok(output) = listener.recv().await {
            let mut buffer = vec![output];
            while let Ok(output) = listener.try_recv() {
                buffer.push(output);
            }
            Ok(buffer)
        } else {
            Err(test_rpc::Error::Logger(
                test_rpc::logging::Error::StandardOutput,
            ))
        }
    }

    async fn try_poll_output(
        self,
        _: context::Context,
    ) -> Result<Vec<test_rpc::logging::Output>, test_rpc::Error> {
        let mut listener = LOGGER.0.lock().await;
        match listener.try_recv() {
            Ok(output) => {
                let mut buffer = vec![output];
                while let Ok(output) = listener.try_recv() {
                    buffer.push(output);
                }
                Ok(buffer)
            }
            Err(TryRecvError::Empty) => Ok(Vec::new()),
            Err(_) => Err(test_rpc::Error::Logger(
                test_rpc::logging::Error::StandardOutput,
            )),
        }
    }

    async fn get_nymvpn_app_logs(self, _: context::Context) -> test_rpc::logging::LogOutput {
        logging::get_nym_app_logs().await
    }

    async fn restart_nymvpn_daemon(self, _: context::Context) -> Result<(), test_rpc::Error> {
        #[cfg(target_os = "linux")]
        {
            sys::restart_app(NYM_SYSTEMD_SERVICE_NAME).await
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(test_rpc::Error::TargetNotImplemented)
        }
    }

    /// Stop the Mullvad VPN application.
    async fn stop_nymvpn_daemon(self, _: context::Context) -> Result<(), test_rpc::Error> {
        #[cfg(target_os = "linux")]
        {
            sys::stop_app(NYM_SYSTEMD_SERVICE_NAME).await
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(test_rpc::Error::TargetNotImplemented)
        }
    }

    /// Start the Mullvad VPN application.
    async fn start_nymvpn_daemon(self, _: context::Context) -> Result<(), test_rpc::Error> {
        #[cfg(target_os = "linux")]
        {
            sys::start_app(NYM_SYSTEMD_SERVICE_NAME).await
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(test_rpc::Error::TargetNotImplemented)
        }
    }

    /// Disable the Mullvad VPN system service.
    async fn disable_nymvpn_daemon(self, _: context::Context) -> Result<(), test_rpc::Error> {
        #[cfg(not(target_os = "windows"))]
        {
            log::warn!("disable_mullvad_daemon is only implemented on Windows");
            return Err(test_rpc::Error::Syscall);
        }
        #[cfg(target_os = "windows")]
        {
            sys::disable_system_service_startup().await
        }
    }

    async fn enable_nymvpn_daemon(self, _: context::Context) -> Result<(), test_rpc::Error> {
        #[cfg(not(target_os = "windows"))]
        {
            log::warn!("enable_mullvad_daemon is only implemented on Windows");
            return Err(test_rpc::Error::Syscall);
        }
        #[cfg(target_os = "windows")]
        {
            sys::enable_system_service_startup().await
        }
    }

    async fn set_daemon_log_level(
        self,
        _: context::Context,
        verbosity_level: test_rpc::nym_daemon::Verbosity,
    ) -> Result<(), test_rpc::Error> {
        #[cfg(target_os = "linux")]
        {
            use crate::sys::NYM_VPN_SYSTEMD_OVERRIDE_FILE;
            sys::set_daemon_log_level(
                verbosity_level,
                NYM_SYSTEMD_SERVICE_NAME,
                NYM_VPN_SYSTEMD_OVERRIDE_FILE,
            )
            .await
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = verbosity_level;
            Err(test_rpc::Error::TargetNotImplemented)
        }
    }

    async fn set_daemon_environment(
        self,
        _: context::Context,
        env: HashMap<String, String>,
    ) -> Result<(), test_rpc::Error> {
        #[cfg(target_os = "linux")]
        {
            use crate::sys::NYM_VPN_SYSTEMD_OVERRIDE_FILE;
            sys::set_daemon_environment(
                env,
                NYM_SYSTEMD_SERVICE_NAME,
                NYM_VPN_SYSTEMD_OVERRIDE_FILE,
            )
            .await
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = env;
            Err(test_rpc::Error::TargetNotImplemented)
        }
    }

    async fn get_daemon_environment(
        self,
        _: context::Context,
    ) -> Result<HashMap<String, String>, test_rpc::Error> {
        #[cfg(target_os = "linux")]
        {
            use crate::sys::NYM_VPN_SYSTEMD_OVERRIDE_FILE;
            sys::get_daemon_environment(NYM_VPN_SYSTEMD_OVERRIDE_FILE).await
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(test_rpc::Error::TargetNotImplemented)
        }
    }

    async fn copy_file(
        self,
        _: context::Context,
        src: String,
        dest: String,
    ) -> Result<(), test_rpc::Error> {
        tokio::fs::copy(&src, &dest).await.map_err(|error| {
            log::error!("Failed to copy \"{src}\" to \"{dest}\": {error}");
            test_rpc::Error::Syscall
        })?;
        Ok(())
    }

    /// Write a slice as the entire contents of a file.
    ///
    /// See the documention of [`tokio::fs::write`] for details of the behavior.
    async fn write_file(
        self,
        _: context::Context,
        dest: PathBuf,
        bytes: Vec<u8>,
    ) -> Result<(), test_rpc::Error> {
        tokio::fs::write(&dest, bytes).await.map_err(|error| {
            log::error!(
                "Failed to write to \"{dest}\": {error}",
                dest = dest.display()
            );
            test_rpc::Error::Syscall
        })?;
        Ok(())
    }

    async fn reboot(self, _: context::Context) -> Result<(), test_rpc::Error> {
        sys::reboot()
    }

    async fn spawn(self, _: context::Context, opts: SpawnOpts) -> Result<u32, test_rpc::Error> {
        let mut cmd = Command::new(&opts.path);
        cmd.args(&opts.args);

        // Make sure that PATH is updated
        // TODO: We currently do not need this on non-Windows
        #[cfg(target_os = "windows")]
        cmd.env("PATH", sys::get_system_path_var()?);

        cmd.envs(opts.env);

        if opts.attach_stdin {
            cmd.stdin(Stdio::piped());
        } else {
            cmd.stdin(Stdio::null());
        }

        if opts.attach_stdout {
            cmd.stdout(Stdio::piped());
        }

        cmd.stderr(Stdio::piped());
        cmd.kill_on_drop(true);

        let mut child = util::as_unprivileged_user(UNPRIVILEGED_USER, || cmd.spawn())
            .map_err(|error| {
                log::error!("Failed to drop privileges: {error}");
                test_rpc::Error::Syscall
            })?
            .map_err(|error| {
                log::error!("Failed to spawn {}: {error}", opts.path);
                test_rpc::Error::Syscall
            })?;

        let pid = child
            .id()
            .expect("Child hasn't been polled to completion yet");

        log::info!("spawned {} (args={:?}) (pid={pid})", opts.path, opts.args);

        let (abort_tx, abort_rx) = oneshot::channel();
        let abort_handle = || {
            let _ = abort_tx.send(());
        };

        let spawned_process = NymSpawnedProcess {
            stdout: child.stdout.take(),
            stdin: child.stdin.take(),
            abort_handle: OnDrop::new(Box::new(abort_handle)),
        };

        let mut state = self.0.lock().await;
        state.spawned_procs.insert(pid, spawned_process);
        drop(state);

        // spawn a task to log child stdout
        if let Some(stderr) = child.stderr.take() {
            task::spawn(async move {
                let mut stderr = BufReader::new(stderr);
                let mut line = String::new();
                loop {
                    match stderr.read_line(&mut line).await {
                        Ok(0) => break,
                        Ok(_) => {
                            let trimmed = line.trim_end_matches(['\r', '\n']);
                            log::info!("child stderr (pid={pid}): {trimmed}");
                            line.clear();
                        }
                        Err(e) => {
                            log::error!("failed to read child stderr (pid={pid}): {e}");
                            break;
                        }
                    }
                }
            });
        }

        // spawn a task to monitor if the child exits
        task::spawn(async move {
            select! {
                result = child.wait().fuse() => match result {
                    Err(e) => {
                        log::error!("failed to await child process (pid={pid}): {e}");
                    }
                    Ok(status) => {
                        log::info!("child process (pid={pid}) exited with status: {status}");
                    }
                },

                _ = abort_rx.fuse() => {
                    if let Err(e) = child.kill().await {
                        log::error!("failed to kill child process (pid={pid}): {e}");
                    }
                }
            }

            let mut state = self.0.lock().await;
            state.spawned_procs.remove(&pid);
        });

        Ok(pid)
    }

    async fn read_child_stdout(
        self,
        _: context::Context,
        pid: u32,
    ) -> Result<Option<String>, test_rpc::Error> {
        let mut state = self.0.lock().await;
        let child = state
            .spawned_procs
            .get_mut(&pid)
            .ok_or(test_rpc::Error::UnknownPid(pid))?;

        let Some(stdout) = child.stdout.as_mut() else {
            return Ok(None);
        };

        let mut buf = vec![0u8; 512];

        let n = select_biased! {
            result = stdout.read(&mut buf).fuse() => result
                .map_err(|e| format!("Failed to read from child stdout: {e}"))
                .map_err(test_rpc::Error::Other)?,

            _ = sleep(Duration::from_millis(500)).fuse() => return Ok(Some(String::new())),
        };

        // check for EOF
        if n == 0 {
            child.stdout = None;
            return Ok(None);
        }

        buf.truncate(n);
        let output = String::from_utf8(buf)
            .map_err(|_| test_rpc::Error::Other("Child wrote non UTF-8 to stdout".into()))?;

        Ok(Some(output))
    }

    async fn write_child_stdin(
        self,
        _: context::Context,
        pid: u32,
        data: String,
    ) -> Result<(), test_rpc::Error> {
        let mut state = self.0.lock().await;
        let child = state
            .spawned_procs
            .get_mut(&pid)
            .ok_or(test_rpc::Error::UnknownPid(pid))?;

        let Some(stdin) = child.stdin.as_mut() else {
            return Err(test_rpc::Error::Other("Child stdin is closed.".into()));
        };

        stdin
            .write_all(data.as_bytes())
            .await
            .map_err(|e| format!("Error writing to child stdin: {e}"))
            .map_err(test_rpc::Error::Other)?;

        log::debug!("wrote {} bytes to pid {pid}", data.len());

        Ok(())
    }

    async fn close_child_stdin(self, _: context::Context, pid: u32) -> Result<(), test_rpc::Error> {
        let mut state = self.0.lock().await;
        let child = state
            .spawned_procs
            .get_mut(&pid)
            .ok_or(test_rpc::Error::UnknownPid(pid))?;

        child.stdin = None;

        Ok(())
    }

    async fn kill_child(self, _: context::Context, pid: u32) -> Result<(), test_rpc::Error> {
        let mut state = self.0.lock().await;
        let child = state
            .spawned_procs
            .remove(&pid)
            .ok_or(test_rpc::Error::UnknownPid(pid))?;

        drop(child); // I swear officer, it's not what you think!

        Ok(())
    }

    async fn get_os_version(self, _: context::Context) -> Result<OsVersion, test_rpc::Error> {
        #[cfg(target_os = "linux")]
        {
            sys::get_os_version()
        }
        #[cfg(target_os = "macos")]
        {
            sys::get_os_version()
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err(test_rpc::Error::TargetNotImplemented)
        }
    }

    #[cfg_attr(not(target_os = "macos"), allow(unused_variables))]
    async fn ifconfig_alias_add(
        self,
        _: context::Context,
        interface: String,
        alias: IpAddr,
    ) -> Result<(), test_rpc::Error> {
        #[cfg(not(target_os = "macos"))]
        return Err(test_rpc::Error::TargetNotImplemented);

        #[cfg(target_os = "macos")]
        {
            let output = Command::new("ifconfig")
                .args([&interface, "alias", &alias.to_string()])
                .output()
                .await
                .map_err(|e| test_rpc::Error::Other(format!("Failed to run ifconfig: {e:#}")))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(test_rpc::Error::Other(format!(
                    "ifconfig alias add failed: {stderr}"
                )));
            }
            Ok(())
        }
    }

    #[cfg_attr(not(target_os = "macos"), allow(unused_variables))]
    async fn ifconfig_alias_remove(
        self,
        _: context::Context,
        interface: String,
        alias: IpAddr,
    ) -> Result<(), test_rpc::Error> {
        #[cfg(not(target_os = "macos"))]
        return Err(test_rpc::Error::TargetNotImplemented);

        #[cfg(target_os = "macos")]
        {
            let output = Command::new("ifconfig")
                .args([&interface, "-alias", &alias.to_string()])
                .output()
                .await
                .map_err(|e| test_rpc::Error::Other(format!("Failed to run ifconfig: {e:#}")))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(test_rpc::Error::Other(format!(
                    "ifconfig alias remove failed: {stderr}"
                )));
            }
            Ok(())
        }
    }
}

/// Poll a guest-local daemon reader every [`OBSERVE_POLL_INTERVAL`] until `accept` matches
/// or `timeout_ms` elapses, returning a single [`WaitOutcome`]. All polling stays on the
/// local UDS; only the one outcome crosses the serial link. Transient read errors and
/// per-read timeouts ([`OBSERVE_READ_TIMEOUT`]) are logged and retried until the deadline
/// so a brief daemon blip or hung call does not stall past `timeout_ms`.
async fn wait_for_observed<S, Fut>(
    timeout_ms: u64,
    mut read: impl FnMut() -> Fut,
    accept: impl Fn(&S) -> bool,
) -> Result<WaitOutcome<S>, test_rpc::Error>
where
    Fut: Future<Output = Result<S, test_rpc::Error>>,
{
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    let mut last_observed = None;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(WaitOutcome::TimedOut { last_observed });
        }

        let read_budget = OBSERVE_READ_TIMEOUT.min(remaining);
        match tokio::time::timeout(read_budget, read()).await {
            Ok(Ok(state)) => {
                if accept(&state) {
                    return Ok(WaitOutcome::Reached(state));
                }
                last_observed = Some(state);
            }
            Ok(Err(error)) => log::warn!("observed-state read failed mid-wait: {error}"),
            Err(_) => log::warn!(
                "observed-state read timed out after {}ms; retrying until wait deadline",
                read_budget.as_millis()
            ),
        }

        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(WaitOutcome::TimedOut { last_observed });
        }
        sleep(OBSERVE_POLL_INTERVAL.min(remaining)).await;
    }
}

pub(crate) fn observed_tunnel_type(tunnel_type: TunnelType) -> ObservedTunnelType {
    match tunnel_type {
        TunnelType::Mixnet => ObservedTunnelType::Mixnet,
        TunnelType::Wireguard => ObservedTunnelType::Wireguard,
    }
}

pub(crate) fn observed_tunnel_state(state: TunnelState) -> ObservedTunnelState {
    match state {
        TunnelState::Connected { connection_data } => ObservedTunnelState::Connected {
            tunnel_type: observed_tunnel_type(connection_data.tunnel.tunnel_type()),
        },
        TunnelState::Disconnected => ObservedTunnelState::Disconnected,
        TunnelState::Connecting { .. } => ObservedTunnelState::Connecting,
        TunnelState::Disconnecting { .. } => ObservedTunnelState::Disconnecting,
        TunnelState::Offline { .. } => ObservedTunnelState::Offline,
        TunnelState::Error(reason) => ObservedTunnelState::Error(reason.to_string()),
    }
}

pub(crate) fn observed_account_state(state: AccountControllerState) -> ObservedAccountState {
    match state {
        AccountControllerState::Offline => ObservedAccountState::Offline,
        AccountControllerState::Syncing => ObservedAccountState::Syncing,
        AccountControllerState::LoggedOut => ObservedAccountState::LoggedOut,
        AccountControllerState::ReadyToConnect => ObservedAccountState::ReadyToConnect,
        AccountControllerState::Decentralised => ObservedAccountState::Decentralised,
        AccountControllerState::PendingSubscription => ObservedAccountState::PendingSubscription,
        AccountControllerState::Error(reason) => ObservedAccountState::Error(reason.to_string()),
    }
}

#[cfg(test)]
mod observed_state_tests {
    use super::{observed_account_state, observed_tunnel_state, observed_tunnel_type};
    use nym_vpn_lib_types::{
        AccountControllerErrorStateReason, AccountControllerState, ActionAfterDisconnect,
        ErrorStateReason, EstablishConnectionState, TunnelState, TunnelType,
    };
    use test_rpc::nym_daemon::{ObservedAccountState, ObservedTunnelState, ObservedTunnelType};

    #[test]
    fn maps_tunnel_types() {
        assert_eq!(
            observed_tunnel_type(TunnelType::Mixnet),
            ObservedTunnelType::Mixnet
        );
        assert_eq!(
            observed_tunnel_type(TunnelType::Wireguard),
            ObservedTunnelType::Wireguard
        );
    }

    #[test]
    fn maps_non_connected_tunnel_discriminants() {
        assert_eq!(
            observed_tunnel_state(TunnelState::Disconnected),
            ObservedTunnelState::Disconnected
        );
        assert_eq!(
            observed_tunnel_state(TunnelState::Connecting {
                retry_attempt: 0,
                state: EstablishConnectionState::ResolvingApiAddresses,
                tunnel_type: TunnelType::Mixnet,
                connection_data: None,
            }),
            ObservedTunnelState::Connecting
        );
        assert_eq!(
            observed_tunnel_state(TunnelState::Disconnecting {
                after_disconnect: ActionAfterDisconnect::Nothing,
            }),
            ObservedTunnelState::Disconnecting
        );
        assert_eq!(
            observed_tunnel_state(TunnelState::Offline { reconnect: false }),
            ObservedTunnelState::Offline
        );
        assert!(matches!(
            observed_tunnel_state(TunnelState::Error(ErrorStateReason::SetFirewallPolicy)),
            ObservedTunnelState::Error(_)
        ));
    }

    #[test]
    fn maps_all_account_discriminants() {
        let cases = [
            (
                AccountControllerState::Offline,
                ObservedAccountState::Offline,
            ),
            (
                AccountControllerState::Syncing,
                ObservedAccountState::Syncing,
            ),
            (
                AccountControllerState::LoggedOut,
                ObservedAccountState::LoggedOut,
            ),
            (
                AccountControllerState::ReadyToConnect,
                ObservedAccountState::ReadyToConnect,
            ),
            (
                AccountControllerState::Decentralised,
                ObservedAccountState::Decentralised,
            ),
            (
                AccountControllerState::PendingSubscription,
                ObservedAccountState::PendingSubscription,
            ),
        ];
        for (input, expected) in cases {
            assert_eq!(observed_account_state(input), expected);
        }
        assert!(matches!(
            observed_account_state(AccountControllerState::Error(
                AccountControllerErrorStateReason::InactiveSubscription
            )),
            ObservedAccountState::Error(_)
        ));
    }
}

#[cfg(test)]
mod wait_for_observed_tests {
    use super::wait_for_observed;
    use std::{cell::RefCell, collections::VecDeque};
    use test_rpc::{
        Error,
        nym_daemon::{
            ObservedTunnelState, ObservedTunnelStateKind, ObservedTunnelType, WaitOutcome,
        },
    };

    fn reader(
        reads: impl IntoIterator<Item = Result<ObservedTunnelState, Error>>,
    ) -> RefCell<VecDeque<Result<ObservedTunnelState, Error>>> {
        RefCell::new(reads.into_iter().collect())
    }

    fn wants_connected(state: &ObservedTunnelState) -> bool {
        ObservedTunnelStateKind::Connected.matches(state)
    }

    #[tokio::test(start_paused = true)]
    async fn returns_reached_when_target_matches() {
        let reads = reader([
            Ok(ObservedTunnelState::Connecting),
            Ok(ObservedTunnelState::Connected {
                tunnel_type: ObservedTunnelType::Wireguard,
            }),
        ]);

        let outcome = wait_for_observed(
            60_000,
            || {
                let next = reads
                    .borrow_mut()
                    .pop_front()
                    .unwrap_or(Ok(ObservedTunnelState::Disconnected));
                async move { next }
            },
            wants_connected,
        )
        .await
        .expect("wait must not error");

        assert_eq!(
            outcome,
            WaitOutcome::Reached(ObservedTunnelState::Connected {
                tunnel_type: ObservedTunnelType::Wireguard,
            })
        );
    }

    #[tokio::test(start_paused = true)]
    async fn times_out_reporting_last_observed() {
        let reads = reader([Ok(ObservedTunnelState::Connecting)]);

        let outcome = wait_for_observed(
            500,
            || {
                let next = reads
                    .borrow_mut()
                    .pop_front()
                    .unwrap_or(Ok(ObservedTunnelState::Disconnected));
                async move { next }
            },
            wants_connected,
        )
        .await
        .expect("timeout is a successful single reply, not an error");

        assert_eq!(
            outcome,
            WaitOutcome::TimedOut {
                last_observed: Some(ObservedTunnelState::Disconnected),
            }
        );
    }

    #[tokio::test(start_paused = true)]
    async fn retries_past_a_transient_read_error() {
        let reads = reader([
            Err(Error::DaemonRpc("transient".into())),
            Ok(ObservedTunnelState::Connected {
                tunnel_type: ObservedTunnelType::Mixnet,
            }),
        ]);

        let outcome = wait_for_observed(
            60_000,
            || {
                let next = reads
                    .borrow_mut()
                    .pop_front()
                    .unwrap_or(Ok(ObservedTunnelState::Disconnected));
                async move { next }
            },
            wants_connected,
        )
        .await
        .expect("wait must recover from a transient error");

        assert_eq!(
            outcome,
            WaitOutcome::Reached(ObservedTunnelState::Connected {
                tunnel_type: ObservedTunnelType::Mixnet,
            })
        );
    }

    #[tokio::test(start_paused = true)]
    async fn honors_deadline_when_a_read_hangs() {
        let started = tokio::time::Instant::now();
        let outcome = wait_for_observed(
            1_000,
            std::future::pending::<Result<ObservedTunnelState, Error>>,
            wants_connected,
        )
        .await
        .expect("a hung read must time out as WaitOutcome, not an error");

        assert_eq!(
            outcome,
            WaitOutcome::TimedOut {
                last_observed: None
            }
        );
        assert_eq!(started.elapsed(), std::time::Duration::from_millis(1_000));
    }
}
