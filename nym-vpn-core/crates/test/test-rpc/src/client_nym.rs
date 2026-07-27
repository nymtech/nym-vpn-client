// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::HashMap,
    path::Path,
    time::{Duration, SystemTime},
};

use crate::nym_daemon::ServiceStatus;

use super::*;

const INSTALL_TIMEOUT: Duration = Duration::from_secs(300);
const REBOOT_TIMEOUT: Duration = Duration::from_secs(60);
/// How long to wait before proceeding after a reboot and a connection to the test-runner has been
/// re-established
const POST_REBOOT_GRACE_PERIOD: Duration = Duration::from_secs(5);
const LOG_LEVEL_TIMEOUT: Duration = Duration::from_secs(60);
const DAEMON_RESTART_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub struct NymServiceClient {
    connection_handle: transport::ConnectionHandle,
    client: service::ServiceClient,
}

// The pattern is:
// #[tarpc::service] generates client code
// -> NymServiceClient wrapper provides ergonomic API
// -> server implements the trait.
// TODO everything basically copied from the mullvad client code
// then gradually modified to suit our needs
impl NymServiceClient {
    pub fn new(
        connection_handle: transport::ConnectionHandle,
        transport: tarpc::transport::channel::UnboundedChannel<
            tarpc::Response<service::ServiceResponse>,
            tarpc::ClientMessage<service::ServiceRequest>,
        >,
    ) -> Self {
        Self {
            connection_handle,
            client: super::service::ServiceClient::new(tarpc::client::Config::default(), transport)
                .spawn(),
        }
    }

    /// Install app package.
    pub async fn install_app(&self, package_path: package::Package) -> Result<(), Error> {
        log::info!("Installing the app...");
        let mut ctx = tarpc::context::current();
        ctx.deadline = SystemTime::now().checked_add(INSTALL_TIMEOUT).unwrap();

        self.client
            .install_app(ctx, package_path)
            .await
            .map_err(Error::Tarpc)?
    }

    /// Remove app package.
    pub async fn uninstall_app(&self, env: HashMap<String, String>) -> Result<(), Error> {
        let mut ctx = tarpc::context::current();
        ctx.deadline = SystemTime::now().checked_add(INSTALL_TIMEOUT).unwrap();

        self.client.uninstall_app(ctx, env).await?
    }

    /// Execute a program with additional environment-variables set.
    pub async fn exec_env<
        I: IntoIterator<Item = T>,
        M: IntoIterator<Item = (K, T)>,
        T: AsRef<str>,
        K: AsRef<str>,
    >(
        &self,
        path: T,
        args: I,
        env: M,
    ) -> Result<ExecResult, Error> {
        let mut ctx = tarpc::context::current();
        ctx.deadline = SystemTime::now().checked_add(INSTALL_TIMEOUT).unwrap();
        self.client
            .exec(
                ctx,
                path.as_ref().to_string(),
                args.into_iter().map(|v| v.as_ref().to_string()).collect(),
                env.into_iter()
                    .map(|(k, v)| (k.as_ref().to_string(), v.as_ref().to_string()))
                    .collect(),
            )
            .await?
    }

    /// Execute a program.
    pub async fn exec<I: IntoIterator<Item = T>, T: AsRef<str>>(
        &self,
        path: T,
        args: I,
    ) -> Result<ExecResult, Error> {
        let env: [(&str, T); 0] = [];
        self.exec_env(path, args, env).await
    }

    /// Get the output of the runners stdout logs since the last time this function was called.
    /// Block if there is no output until some output is provided by the runner.
    pub async fn poll_output(&self) -> Result<Vec<logging::Output>, Error> {
        self.client.poll_output(tarpc::context::current()).await?
    }

    /// Get the output of the runners stdout logs since the last time this function was called.
    /// Block if there is no output until some output is provided by the runner.
    pub async fn try_poll_output(&self) -> Result<Vec<logging::Output>, Error> {
        self.client
            .try_poll_output(tarpc::context::current())
            .await?
    }

    pub async fn get_nym_app_logs(&self) -> Result<logging::LogOutput, Error> {
        self.client
            .get_nymvpn_app_logs(tarpc::context::current())
            .await
            .map_err(Error::Tarpc)
    }

    /// Wait for the Mullvad service to enter a specified state. The state is inferred from the
    /// presence of a named pipe or UDS, not the actual system service state.
    pub async fn nymvpn_daemon_wait_for_state(
        &self,
        accept_state_fn: impl Fn(ServiceStatus) -> bool,
    ) -> Result<nym_daemon::ServiceStatus, Error> {
        const MAX_ATTEMPTS: usize = 10;
        const POLL_INTERVAL: Duration = Duration::from_secs(3);

        for _ in 0..MAX_ATTEMPTS {
            let last_state = self.nymvpn_daemon_get_status().await?;
            match accept_state_fn(last_state) {
                true => return Ok(last_state),
                false => tokio::time::sleep(POLL_INTERVAL).await,
            }
        }
        Err(Error::Timeout)
    }

    /// Return status of the system service. The state is inferred from the presence of
    /// a named pipe or UDS, not the actual system service state.
    pub async fn nymvpn_daemon_get_status(&self) -> Result<nym_daemon::ServiceStatus, Error> {
        self.client
            .nymvpn_daemon_get_status(tarpc::context::current())
            .await
            .map_err(Error::Tarpc)
    }

    /// Tunnel discriminant from guest-local daemon UDS (not serial-forwarded gRPC).
    pub async fn get_observed_tunnel_state(
        &self,
    ) -> Result<nym_daemon::ObservedTunnelState, Error> {
        self.client
            .get_observed_tunnel_state(tarpc::context::current())
            .await
            .map_err(Error::Tarpc)?
    }

    /// Account discriminant from guest-local daemon UDS (not serial-forwarded gRPC).
    pub async fn get_observed_account_state(
        &self,
    ) -> Result<nym_daemon::ObservedAccountState, Error> {
        self.client
            .get_observed_account_state(tarpc::context::current())
            .await
            .map_err(Error::Tarpc)?
    }

    /// Return the version string as reported by `mullvad --version`.
    ///
    /// TODO: Replace with nicer version type.
    pub async fn nymvpn_daemon_version(&self) -> Result<String, Error> {
        self.client
            .nymvpn_version(tarpc::context::current())
            .await
            .map_err(Error::Tarpc)?
    }

    /// Returns all nym app files, directories, and other data found on the system.
    pub async fn find_nymvpnd_app_traces(&self) -> Result<Vec<AppTrace>, Error> {
        self.client
            .find_nymvpn_app_traces(tarpc::context::current())
            .await?
    }

    /// Returns path of Nym app cache directorie on the test runner.
    pub async fn find_nymvpnd_cache_dir(&self) -> Result<PathBuf, Error> {
        self.client
            .get_nym_app_cache_dir(tarpc::context::current())
            .await?
    }

    /// Send TCP packet
    pub async fn send_tcp(
        &self,
        interface: Option<String>,
        bind_addr: SocketAddr,
        destination: SocketAddr,
    ) -> Result<(), Error> {
        self.client
            .send_tcp(tarpc::context::current(), interface, bind_addr, destination)
            .await?
    }

    /// Send UDP packet
    pub async fn send_udp(
        &self,
        interface: Option<String>,
        bind_addr: SocketAddr,
        destination: SocketAddr,
    ) -> Result<(), Error> {
        self.client
            .send_udp(tarpc::context::current(), interface, bind_addr, destination)
            .await?
    }

    /// Send ICMP
    pub async fn send_ping(
        &self,
        destination: IpAddr,
        interface: Option<String>,
        size: usize,
    ) -> Result<(), Error> {
        self.client
            .send_ping(tarpc::context::current(), destination, interface, size)
            .await?
    }

    /// Fetch the current location.
    pub async fn geoip_lookup(&self) -> Result<AmIMullvad, Error> {
        self.client.geoip_lookup(tarpc::context::current()).await?
    }

    /// Returns the IP of the given interface.
    pub async fn get_interface_ip(&self, interface: String) -> Result<IpAddr, Error> {
        self.client
            .get_interface_ip(tarpc::context::current(), interface)
            .await?
    }

    /// Returns the MTU of the given interface.
    pub async fn get_interface_mtu(&self, interface: String) -> Result<u16, Error> {
        self.client
            .get_interface_mtu(tarpc::context::current(), interface)
            .await?
    }

    /// Returns the MAC address of the given interface.
    pub async fn get_interface_mac(&self, interface: String) -> Result<Option<[u8; 6]>, Error> {
        self.client
            .get_interface_mac(tarpc::context::current(), interface)
            .await?
    }

    /// Returns the name of the default non-tunnel interface
    pub async fn get_default_interface(&self) -> Result<String, Error> {
        self.client
            .get_default_interface(tarpc::context::current())
            .await?
    }

    pub async fn resolve_hostname(&self, hostname: String) -> Result<Vec<SocketAddr>, Error> {
        self.client
            .resolve_hostname(tarpc::context::current(), hostname)
            .await?
    }

    /// Start forwarding TCP from a server listening on `bind_addr` to the given address, and return
    /// a handle that closes the server when dropped
    pub async fn start_tcp_forward(
        &self,
        bind_addr: SocketAddr,
        via_addr: SocketAddr,
    ) -> Result<crate::net::SockHandle, Error> {
        crate::net::SockHandle::start_tcp_forward(self.client.clone(), bind_addr, via_addr).await
    }

    /// Restarts the app.
    ///
    /// Shuts down a running app, making it disconnect from any current tunnel
    /// connection before starting the app again.
    ///
    /// # Note
    /// This function will return *after* the app is running again, thus
    /// blocking execution until then.
    pub async fn restart_nymvpn_daemon(&self) -> Result<(), Error> {
        let _ = self
            .client
            .restart_nymvpn_daemon(tarpc::context::current())
            .await?;
        Ok(())
    }

    /// Stop the app.
    ///
    /// Shuts down a running app, making it disconnect from any current tunnel
    /// connection and making it write to caches.
    ///
    /// # Note
    /// This function will return *after* the app has been stopped, thus
    /// blocking execution until then.
    pub async fn stop_nymvpn_daemon(&self) -> Result<(), Error> {
        let mut ctx = tarpc::context::current();
        ctx.deadline = SystemTime::now()
            .checked_add(DAEMON_RESTART_TIMEOUT)
            .unwrap();
        let _ = self.client.stop_nymvpn_daemon(ctx).await?;
        Ok(())
    }

    /// Start the app.
    ///
    /// # Note
    /// This function will return *after* the app has been started, thus
    /// blocking execution until then.
    pub async fn start_nymvpn_daemon(&self) -> Result<(), Error> {
        let _ = self
            .client
            .start_nymvpn_daemon(tarpc::context::current())
            .await?;
        Ok(())
    }

    /// Enable the daemon system service.
    ///
    /// Does *not* start a stopped app. See [start_mullvad_daemon].
    pub async fn enable_nymvpn_daemon(&self) -> Result<(), Error> {
        let mut ctx = tarpc::context::current();
        ctx.deadline = SystemTime::now()
            .checked_add(DAEMON_RESTART_TIMEOUT)
            .unwrap();
        self.client
            .enable_nymvpn_daemon(ctx)
            .await
            .map_err(Error::Tarpc)??;
        Ok(())
    }

    /// Disable the daemon system service. *Current only works on Windows*.
    ///
    /// This will not stop the daemon system service, but it will prevent it from starting
    /// automatically on system boot.
    ///
    /// Note that if the daemon is also stopped, using [stop_mullvad_daemon], it will
    /// not be possible to start it again until it is enabled again using
    /// [enable_mullvad_daemon].
    pub async fn disable_nymvpn_daemon(&self) -> Result<(), Error> {
        let mut ctx = tarpc::context::current();
        ctx.deadline = SystemTime::now()
            .checked_add(DAEMON_RESTART_TIMEOUT)
            .unwrap();
        self.client
            .disable_nymvpn_daemon(ctx)
            .await
            .map_err(Error::Tarpc)??;
        Ok(())
    }

    pub async fn set_daemon_log_level(
        &self,
        verbosity_level: nym_daemon::Verbosity,
    ) -> Result<(), Error> {
        let mut ctx = tarpc::context::current();
        ctx.deadline = SystemTime::now().checked_add(LOG_LEVEL_TIMEOUT).unwrap();
        self.client
            .set_daemon_log_level(ctx, verbosity_level)
            .await??;

        self.nymvpn_daemon_wait_for_state(|state| state == ServiceStatus::Running)
            .await?;

        Ok(())
    }

    /// Set environment variables specified by `env` and restart the Mullvad daemon.
    ///
    /// # Returns
    /// - `Result::Ok` if the daemon was successfully restarted.
    /// - `Result::Err(Error)` if the daemon could not be restarted and is thus no longer running.
    pub async fn set_daemon_environment<Env, K, V>(&self, env: Env) -> Result<(), Error>
    where
        Env: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let mut ctx = tarpc::context::current();
        ctx.deadline = SystemTime::now().checked_add(LOG_LEVEL_TIMEOUT).unwrap();
        let env = env.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        self.client.set_daemon_environment(ctx, env).await??;

        self.nymvpn_daemon_wait_for_state(|state| state == ServiceStatus::Running)
            .await?;

        Ok(())
    }

    /// Get the current daemon's environment variables.
    ///
    /// # Returns
    /// - `Result::Ok(env)` if the current environment variables could be read.
    /// - `Result::Err(Error)` if communication with the daemon failed or the environment values
    ///   could not be parsed.
    ///   This reads overrides to default env variables defined for that systemd service, not the process'
    ///   currently set environment.
    pub async fn get_daemon_environment_overrides(&self) -> Result<HashMap<String, String>, Error> {
        let env = self
            .client
            .get_daemon_environment(tarpc::context::current())
            .await??;
        Ok(env)
    }

    pub async fn copy_file(&self, src: String, dest: String) -> Result<(), Error> {
        log::debug!("Copying \"{src}\" to \"{dest}\"");
        self.client
            .copy_file(tarpc::context::current(), src, dest)
            .await?
    }

    pub async fn write_file(&self, dest: impl AsRef<Path>, bytes: Vec<u8>) -> Result<(), Error> {
        log::debug!(
            "Writing {bytes} bytes to \"{file}\"",
            bytes = bytes.len(),
            file = dest.as_ref().display()
        );
        self.client
            .write_file(
                tarpc::context::current(),
                dest.as_ref().to_path_buf(),
                bytes,
            )
            .await?
    }

    /// Reboot the testing VM. The VM should be completely rebooted and responsive when this
    /// future completes.
    pub async fn reboot(&mut self) -> Result<(), Error> {
        log::debug!("Rebooting server");

        let mut ctx = tarpc::context::current();
        ctx.deadline = SystemTime::now().checked_add(REBOOT_TIMEOUT).unwrap();

        self.client.reboot(ctx).await??;
        self.connection_handle.reset_connected_state().await;
        self.connection_handle.wait_for_server().await?;

        tokio::time::sleep(POST_REBOOT_GRACE_PERIOD).await;

        Ok(())
    }

    pub async fn spawn(&self, opts: SpawnOpts) -> Result<u32, Error> {
        self.client.spawn(tarpc::context::current(), opts).await?
    }

    pub async fn read_child_stdout(&self, pid: u32) -> Result<Option<String>, Error> {
        self.client
            .read_child_stdout(tarpc::context::current(), pid)
            .await?
    }

    pub async fn write_child_stdin(&self, pid: u32, data: String) -> Result<(), Error> {
        self.client
            .write_child_stdin(tarpc::context::current(), pid, data)
            .await?
    }

    pub async fn close_child_stdin(&self, pid: u32) -> Result<(), Error> {
        self.client
            .close_child_stdin(tarpc::context::current(), pid)
            .await?
    }

    pub async fn get_os_version(&self) -> Result<meta::OsVersion, Error> {
        self.client
            .get_os_version(tarpc::context::current())
            .await?
    }

    pub async fn ifconfig_alias_add(
        &self,
        interface: impl Into<String>,
        alias: impl Into<IpAddr>,
    ) -> Result<(), Error> {
        self.client
            .ifconfig_alias_add(tarpc::context::current(), interface.into(), alias.into())
            .await?
    }

    pub async fn ifconfig_alias_remove(
        &self,
        interface: impl Into<String>,
        alias: impl Into<IpAddr>,
    ) -> Result<(), Error> {
        self.client
            .ifconfig_alias_remove(tarpc::context::current(), interface.into(), alias.into())
            .await?
    }
}
