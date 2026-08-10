// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#![allow(clippy::undocumented_unsafe_blocks)] // Remove me if you dare.

mod driver;
mod path_monitor;
mod service;
mod volume_monitor;
mod windows;

pub use service::{install_driver_service, uninstall_driver_service};

use crate::{SplitTunnelErrorCause, VpnInterface};
use nym_common::ErrorExt;
use nym_routing::{RouteManagerHandle, get_best_default_route};
use nym_windows::{
    io::Overlapped,
    net::{AddressFamily, get_ip_address_for_interface},
    sync::Event,
};
use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    os::windows::io::AsRawHandle,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock, mpsc as sync_mpsc},
    time::Duration,
};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use windows_sys::Win32::Foundation::ERROR_OPERATION_ABORTED;

const DRIVER_EVENT_BUFFER_SIZE: usize = 2048;

/// Cloneable handle for interacting with the split tunnel module.
#[derive(Debug, Clone)]
pub struct SplitTunnelHandle {
    tx: mpsc::UnboundedSender<Message>,
}

impl SplitTunnelHandle {
    /// Set paths to exclude
    pub async fn set_exclude_paths(
        &self,
        paths: HashSet<PathBuf>,
        hybrid_paths: HashSet<PathBuf>,
    ) -> Result<(), Error> {
        let (result_tx, result_rx) = oneshot::channel();
        let _ = self.tx.send(Message::SetExcludePaths {
            result_tx,
            paths,
            hybrid_paths,
        });
        result_rx.await.map_err(|_| Error::Unavailable)?
    }

    /// Set VPN tunnel interface
    pub async fn set_tunnel(&self, vpn_interface: VpnInterface) -> Result<(), Error> {
        let (result_tx, result_rx) = oneshot::channel();
        let _ = self.tx.send(Message::SetTunnel {
            result_tx,
            vpn_interface,
        });
        result_rx.await.map_err(|_| Error::Unavailable)?
    }

    /// Forget the VPN tunnel interface. This destroys the split tunneling interface when it is
    /// active.
    pub async fn reset_tunnel(&self) -> Result<(), Error> {
        let (result_tx, result_rx) = oneshot::channel();
        let _ = self.tx.send(Message::ResetTunnel { result_tx });
        result_rx.await.map_err(|_| Error::Unavailable)?
    }
}

pub struct SplitTunnel {
    _error_handler: Box<dyn Fn(SplitTunnelErrorCause) + Send>,
    rx: mpsc::UnboundedReceiver<Message>,
    shutdown_token: CancellationToken,
    state: State,
}

impl SplitTunnel {
    pub async fn spawn<F>(
        route_manager: RouteManagerHandle,
        shutdown_token: CancellationToken,
        error_handler: F,
    ) -> (SplitTunnelHandle, JoinHandle<()>)
    where
        F: Fn(SplitTunnelErrorCause) + Send + 'static,
    {
        let state = InitializedSplitTunnel::new(route_manager)
            .map(State::Initialized)
            .unwrap_or_else(|error| {
                tracing::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to initialize split tunnel")
                );
                State::Failed
            });

        let (tx, rx) = mpsc::unbounded_channel();

        let split_tunnel = Self {
            _error_handler: Box::new(error_handler),
            rx,
            shutdown_token,
            state,
        };

        let join_handle = tokio::spawn(split_tunnel.run());

        (SplitTunnelHandle { tx }, join_handle)
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                // Handle messages
                message = self.rx.recv() => {
                    let Some(message) = message else {
                        break;
                    };
                    match message {
                        Message::SetExcludePaths {
                            result_tx,
                            paths,
                            hybrid_paths,
                        } => {
                            let _ = result_tx
                                .send(self.set_exclude_paths(paths, hybrid_paths));
                        }
                        Message::SetTunnel {
                            result_tx,
                            vpn_interface,
                        } => {
                            let _ = result_tx.send(self.set_tunnel(vpn_interface));
                        }
                        Message::ResetTunnel { result_tx } => {
                            let _ = result_tx.send(self.reset_tunnel());
                        }
                    }
                }

                _ = self.shutdown_token.cancelled() => {
                    break;
                }
            }
        }
    }

    /// Set a list of applications to exclude from the tunnel.
    fn set_exclude_paths(
        &mut self,
        paths: HashSet<PathBuf>,
        hybrid_paths: HashSet<PathBuf>,
    ) -> Result<(), Error> {
        match &mut self.state {
            State::Initialized(initialized) => {
                tracing::debug!("Updating split tunnel excluded paths: {:?}", paths);
                initialized.set_exclude_paths(paths, hybrid_paths)
            }
            State::Failed => {
                // If we return `Error::Unavailable` here, we will break the tunnel connect logic,
                // so instead just pretend everything is OK.
                tracing::debug!("Split tunnel disabled; ignoring excluded paths");
                Ok(())
            }
        }
    }

    /// Instructs the driver to redirect traffic from sockets bound to 0.0.0.0, ::, or the
    /// tunnel addresses (if any) to the default route.
    fn set_tunnel(&mut self, tunnel: VpnInterface) -> Result<(), Error> {
        match &mut self.state {
            State::Initialized(initialized) => {
                tracing::debug!("Setting tunnel: {:?}", tunnel);
                initialized.set_tunnel(tunnel)
            }
            State::Failed => {
                // If we return `Error::Unavailable` here, we will break the tunnel connect logic,
                // so instead just pretend everything is OK.
                tracing::debug!("Split tunnel disabled; ignoring set tunnel request");
                Ok(())
            }
        }
    }

    /// Instructs the driver to stop redirecting tunnel traffic and INADDR_ANY.
    fn reset_tunnel(&mut self) -> Result<(), Error> {
        match &mut self.state {
            State::Initialized(initialized) => {
                tracing::debug!("Resetting tunnel");
                initialized.reset_tunnel()
            }
            State::Failed => {
                // If we return `Error::Unavailable` here, we will break the tunnel connect logic,
                // so instead just pretend everything is OK.
                tracing::debug!("Split tunnel disabled; ignoring reset tunnel request");
                Ok(())
            }
        }
    }
}

enum State {
    Initialized(InitializedSplitTunnel),
    Failed,
}

struct InitializedSplitTunnel {
    request_tx: RequestTx,
    event_thread: Option<std::thread::JoinHandle<()>>,
    quit_event: Arc<Event>,
    _route_manager: RouteManagerHandle,
}

impl InitializedSplitTunnel {
    fn new(route_manager: RouteManagerHandle) -> Result<Self, Error> {
        let excluded_processes = Arc::new(RwLock::new(HashMap::new()));

        let (request_tx, handle) = Self::spawn_request_thread(excluded_processes.clone())?;

        let (event_thread, quit_event) = Self::spawn_event_listener(handle, excluded_processes)?;

        Ok(Self {
            request_tx,
            event_thread: Some(event_thread),
            quit_event,
            _route_manager: route_manager,
        })
    }

    fn spawn_event_listener(
        handle: Arc<driver::DeviceHandle>,
        excluded_processes: Arc<RwLock<HashMap<usize, ExcludedProcess>>>,
    ) -> Result<(std::thread::JoinHandle<()>, Arc<Event>), Error> {
        let mut event_overlapped = Overlapped::new(Some(
            Event::new(true, false).map_err(Error::EventThreadError)?,
        ))
        .map_err(Error::EventThreadError)?;

        let quit_event = Arc::new(Event::new(true, false).map_err(Error::EventThreadError)?);
        let quit_event_copy = quit_event.clone();

        let event_thread = std::thread::spawn(move || {
            tracing::debug!("Starting split tunnel event thread");
            let mut data_buffer = vec![];

            loop {
                // Wait until either the next event is received or the quit event is signaled.
                let (event_id, event_body) = match Self::fetch_next_event(
                    &handle,
                    &quit_event,
                    &mut event_overlapped,
                    &mut data_buffer,
                ) {
                    Ok(EventResult::Event(event_id, event_body)) => (event_id, event_body),
                    Ok(EventResult::Quit) => break,
                    Err(error) => {
                        if error.raw_os_error() == Some(ERROR_OPERATION_ABORTED as i32) {
                            // The driver will normally abort the request if the driver state
                            // is reset. Give the driver service some time to recover before
                            // retrying.
                            std::thread::sleep(Duration::from_millis(500));
                        }
                        continue;
                    }
                };

                Self::handle_event(event_id, event_body, &excluded_processes);
            }

            tracing::debug!("Stopping split tunnel event thread");
        });

        Ok((event_thread, quit_event_copy))
    }

    fn fetch_next_event(
        device: &Arc<driver::DeviceHandle>,
        quit_event: &Event,
        overlapped: &mut Overlapped,
        data_buffer: &mut Vec<u8>,
    ) -> io::Result<EventResult> {
        if unsafe { windows::wait_for_single_object(quit_event, Some(Duration::ZERO)) }.is_ok() {
            return Ok(EventResult::Quit);
        }

        data_buffer.resize(DRIVER_EVENT_BUFFER_SIZE, 0u8);

        unsafe {
            driver::device_io_control_buffer_async(
                device,
                driver::DriverIoctlCode::DequeEvent as u32,
                None,
                data_buffer.as_mut_ptr(),
                u32::try_from(data_buffer.len()).expect("data buffer too large"),
                overlapped.as_mut_ptr(),
            )
        }
        .inspect_err(|error| {
            tracing::error!(
                "{}",
                error.display_chain_with_msg("DeviceIoControl failed to deque event")
            );
        })?;

        let event_objects = [
            overlapped.get_event().unwrap().as_raw_handle(),
            quit_event.as_raw_handle(),
        ];

        let signaled_object =
            unsafe { windows::wait_for_multiple_objects(&event_objects[..], false) }.inspect_err(
                |error| {
                    tracing::error!(
                        "{}",
                        error.display_chain_with_msg("wait_for_multiple_objects failed")
                    );
                },
            )?;

        if signaled_object == quit_event.as_raw_handle() {
            // Quit event was signaled
            return Ok(EventResult::Quit);
        }

        let returned_bytes = windows::get_overlapped_result(device.as_raw_handle(), overlapped)
            .inspect_err(|error| {
                if error.raw_os_error() != Some(ERROR_OPERATION_ABORTED as i32) {
                    tracing::error!(
                        "{}",
                        error.display_chain_with_msg(
                            "get_overlapped_result failed for dequeued event"
                        ),
                    );
                }
            })?;

        data_buffer.truncate(returned_bytes as usize);

        driver::parse_event_buffer(data_buffer)
            .map(|(id, body)| EventResult::Event(id, body))
            .map_err(|error| {
                tracing::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to parse ST event buffer")
                );
                io::Error::other("Failed to parse ST event buffer")
            })
    }

    fn handle_event(
        event_id: driver::EventId,
        event_body: driver::EventBody,
        excluded_processes: &Arc<RwLock<HashMap<usize, ExcludedProcess>>>,
    ) {
        use driver::{EventBody, EventId};

        let event_str = match &event_id {
            EventId::StartSplittingProcess | EventId::ErrorStartSplittingProcess => {
                "Start splitting process"
            }
            EventId::StopSplittingProcess | EventId::ErrorStopSplittingProcess => {
                "Stop splitting process"
            }
            EventId::ErrorMessage => "ErrorMessage",
        };

        match event_body {
            EventBody::SplittingEvent {
                process_id,
                reason,
                image,
            } => {
                let mut pids = excluded_processes.write().unwrap();
                match event_id {
                    EventId::StartSplittingProcess => {
                        if let Some(prev_entry) = pids.get(&process_id) {
                            tracing::error!(
                                "PID collision: {process_id} is already in the list of excluded processes. New image: {:?}. Current image: {:?}",
                                image,
                                prev_entry
                            );
                        }
                        pids.insert(
                            process_id,
                            ExcludedProcess {
                                pid: u32::try_from(process_id).expect("process_id too large"),
                                image: Path::new(&image).to_path_buf(),
                                inherited: reason
                                    .contains(driver::SplittingChangeReason::BY_INHERITANCE),
                            },
                        );
                    }
                    EventId::StopSplittingProcess if pids.remove(&process_id).is_none() => {
                        tracing::error!("Inconsistent process tree: {process_id} was not found");
                    }
                    _ => (),
                }

                tracing::trace!(
                    "{event_str}:\n\tpid: {process_id}\n\treason: {reason:?}\n\timage: {image:?}"
                );
            }
            EventBody::SplittingError { process_id, image } => {
                tracing::error!("FAILED: {event_str}:\n\tpid: {process_id}\n\timage: {image:?}");
            }
            EventBody::ErrorMessage { status, message } => {
                tracing::error!("NTSTATUS {status:#x}: {}", message.to_string_lossy())
            }
        }
    }

    fn spawn_request_thread(
        excluded_processes: Arc<RwLock<HashMap<usize, ExcludedProcess>>>,
    ) -> Result<(RequestTx, Arc<driver::DeviceHandle>), Error> {
        let (tx, rx): (RequestTx, _) = sync_mpsc::channel();
        let (init_tx, init_rx) = sync_mpsc::channel();

        let monitored_paths = Arc::new(Mutex::new((vec![], vec![])));
        let monitored_paths_copy = monitored_paths.clone();

        let (monitor_tx, monitor_rx) = sync_mpsc::channel();

        let path_monitor = path_monitor::PathMonitor::spawn(monitor_tx.clone())
            .map_err(Error::StartPathMonitor)?;
        let volume_monitor = volume_monitor::VolumeMonitor::spawn(
            path_monitor.clone(),
            monitor_tx,
            monitored_paths.clone(),
        );

        std::thread::spawn(move || {
            let init_fn = || {
                service::start_driver_service().map_err(Error::ServiceError)?;
                driver::DeviceHandle::new()
                    .map(Arc::new)
                    .map_err(Error::InitializationError)
            };

            let handle = match init_fn() {
                Ok(handle) => {
                    let _ = init_tx.send(Ok(handle.clone()));
                    handle
                }
                Err(error) => {
                    let _ = unsafe { service::stop_driver_service() };
                    let _ = init_tx.send(Err(error));
                    return;
                }
            };

            let mut previous_addresses = InterfaceAddresses::default();

            while let Ok((request, response_tx)) = rx.recv() {
                let response = match request {
                    Request::SetPaths {
                        paths,
                        hybrid_paths,
                    } => {
                        let mut monitored_paths_guard = monitored_paths.lock().unwrap();

                        let result = if !paths.is_empty() || !hybrid_paths.is_empty() {
                            handle
                                .set_config(&paths, &hybrid_paths)
                                .map_err(Error::SetConfiguration)
                        } else {
                            handle.clear_config().map_err(Error::SetConfiguration)
                        };

                        if result.is_ok() {
                            let all_paths: Vec<OsString> =
                                paths.iter().chain(hybrid_paths.iter()).cloned().collect();
                            if let Err(error) = path_monitor.set_paths(&all_paths) {
                                tracing::error!(
                                    "{}",
                                    error.display_chain_with_msg("Failed to update path monitor")
                                );
                            }
                            *monitored_paths_guard = (paths, hybrid_paths);
                        }

                        result
                    }
                    Request::RegisterIps(mut ips) => {
                        // If there's no real (non-tunnel) route for a given address family,
                        // don't register a tunnel address for it either: otherwise excluded
                        // processes have nowhere to be redirected to for that family and their
                        // traffic falls through to the tunnel's own default route, leaking it.
                        if ips.internet_ipv4.is_none() {
                            ips.tunnel_ipv4 = None;
                        }
                        if ips.internet_ipv6.is_none() {
                            ips.tunnel_ipv6 = None;
                        }
                        if previous_addresses == ips {
                            Ok(())
                        } else {
                            let result = handle
                                .register_ips(
                                    ips.tunnel_ipv4,
                                    ips.tunnel_ipv6,
                                    ips.internet_ipv4,
                                    ips.internet_ipv6,
                                )
                                .map_err(Error::RegisterIps);
                            if result.is_ok() {
                                previous_addresses = ips;
                            }
                            result
                        }
                    }
                    // INVARIANT: This arm will always the terminate the request thread.
                    Request::Stop => {
                        // Start by attempting to reset the driver state. Do this first, since
                        // we'd like to prevent the process monitor from updating `excluded_processes`.
                        // If reset fails, the driver ends up in a "zombie" state. If that happens,
                        // the best we can do is try to clean up as much as possible.
                        let reset_result = handle.reset().map_err(Error::ResetError);

                        *monitored_paths.lock().unwrap() = (vec![], vec![]);
                        excluded_processes.write().unwrap().clear();

                        drop(volume_monitor);
                        if let Err(error) = path_monitor.shutdown() {
                            tracing::error!(
                                "{}",
                                error.display_chain_with_msg("Failed to shut down path monitor")
                            );
                        }

                        // Device handles must be dropped before unloading the driver.
                        // Otherwise, it will fail and time out.
                        drop(handle);

                        // If we failed to reset, make sure to NEVER unload the driver.
                        // See the safety comment on `stop_driver_service`.
                        // Unloading without a reset can trigger a BSOD!
                        let unload_driver = reset_result.is_ok();

                        if unload_driver {
                            tracing::debug!("Stopping ST service");
                            // SAFETY: We have reset the driver before calling this.
                            if let Err(error) = unsafe { service::stop_driver_service() } {
                                tracing::error!(
                                    "{}",
                                    error.display_chain_with_msg("Failed to stop ST service")
                                );
                            }
                        }

                        let _ = response_tx.send(reset_result);
                        break;
                    }
                };
                if response_tx.send(response).is_err() {
                    tracing::error!("A response could not be sent for a completed request");
                }
            }

            tracing::info!("Stopping ST request thread");
        });

        let handle = init_rx
            .recv_timeout(REQUEST_TIMEOUT)
            .map_err(|_| Error::RequestThreadStuck)??;

        let handle_copy = handle.clone();

        std::thread::spawn(move || {
            while let Ok(()) = monitor_rx.recv() {
                let paths_guard = monitored_paths_copy.lock().unwrap();
                let (paths, hybrid_paths) = &*paths_guard;
                let result = if !paths.is_empty() || !hybrid_paths.is_empty() {
                    tracing::debug!("Re-resolving excluded paths");
                    handle_copy.set_config(paths, hybrid_paths)
                } else {
                    drop(paths_guard);
                    continue;
                };
                drop(paths_guard);
                if let Err(error) = result {
                    tracing::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to update excluded paths")
                    );
                }
            }
        });

        Ok((tx, handle))
    }

    fn send_request(&self, request: Request) -> Result<(), Error> {
        Self::send_request_inner(&self.request_tx, request)
    }

    fn send_request_inner(request_tx: &RequestTx, request: Request) -> Result<(), Error> {
        let (response_tx, response_rx) = sync_mpsc::channel();

        request_tx
            .send((request, response_tx))
            .map_err(|_| Error::SplitTunnelDown)?;

        response_rx
            .recv_timeout(REQUEST_TIMEOUT)
            .map_err(|_| Error::RequestThreadStuck)?
    }

    /// Set a list of applications to exclude from the tunnel.
    fn set_exclude_paths(
        &mut self,
        paths: HashSet<PathBuf>,
        hybrid_paths: HashSet<PathBuf>,
    ) -> Result<(), Error> {
        // If a path appears in both sets, favour hybrid_paths and remove it from paths.
        let paths: HashSet<PathBuf> = paths.difference(&hybrid_paths).cloned().collect();
        self.send_request(Request::SetPaths {
            paths: paths
                .into_iter()
                .map(|path| path.into_os_string())
                .collect(),
            hybrid_paths: hybrid_paths
                .into_iter()
                .map(|path| path.into_os_string())
                .collect(),
        })
    }

    /// Instructs the driver to redirect traffic from sockets bound to 0.0.0.0, ::, or the
    /// tunnel addresses (if any) to the default route.
    fn set_tunnel(&mut self, tunnel: VpnInterface) -> Result<(), Error> {
        let addresses = InterfaceAddresses::from_vpn_interface(&tunnel)?;
        self.send_request(Request::RegisterIps(addresses))
    }

    /// Instructs the driver to stop redirecting tunnel traffic and INADDR_ANY.
    fn reset_tunnel(&mut self) -> Result<(), Error> {
        self.send_request(Request::RegisterIps(InterfaceAddresses::default()))
    }
}

impl Drop for InitializedSplitTunnel {
    fn drop(&mut self) {
        if let Some(_event_thread) = self.event_thread.take()
            && let Err(error) = self.quit_event.set()
        {
            tracing::error!(
                "{}",
                error.display_chain_with_msg("Failed to close ST event thread")
            );
            // Not joining `event_thread`: It may be unresponsive.
        }

        if let Err(error) = self.send_request(Request::Stop) {
            tracing::error!(
                "{}",
                error.display_chain_with_msg("Failed to stop ST driver service")
            );
        }
    }
}

#[derive(PartialEq, Eq)]
enum Request {
    SetPaths {
        paths: Vec<OsString>,
        hybrid_paths: Vec<OsString>,
    },
    RegisterIps(InterfaceAddresses),
    Stop,
}
type RequestResponseTx = sync_mpsc::Sender<Result<(), Error>>;
type RequestTx = sync_mpsc::Sender<(Request, RequestResponseTx)>;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Default, PartialEq, Clone, Eq)]
struct InterfaceAddresses {
    tunnel_ipv4: Option<Ipv4Addr>,
    tunnel_ipv6: Option<Ipv6Addr>,
    internet_ipv4: Option<Ipv4Addr>,
    internet_ipv6: Option<Ipv6Addr>,
}

impl InterfaceAddresses {
    fn from_vpn_interface(vpn_interface: &VpnInterface) -> Result<Self, Error> {
        let tunnel_ipv4 = vpn_interface.v4_address;
        let tunnel_ipv6 = vpn_interface.v6_address;

        // Identify IP address that gives us Internet access
        let internet_ipv4 = get_best_default_route(AddressFamily::Ipv4)
            .map_err(Error::ObtainDefaultRoute)?
            .map(|route| {
                get_ip_address_for_interface(AddressFamily::Ipv4, route.iface).map(|ip| match ip {
                    Some(IpAddr::V4(addr)) => Some(addr),
                    Some(_) => unreachable!("wrong address family (expected IPv4)"),
                    None => {
                        tracing::warn!("No IPv4 address was found for the default route interface");
                        None
                    }
                })
            })
            .transpose()
            .map_err(Error::LuidToIp)?
            .flatten();

        let internet_ipv6 = get_best_default_route(AddressFamily::Ipv6)
            .map_err(Error::ObtainDefaultRoute)?
            .map(|route| {
                get_ip_address_for_interface(AddressFamily::Ipv6, route.iface).map(|ip| match ip {
                    Some(IpAddr::V6(addr)) => Some(addr),
                    Some(_) => unreachable!("wrong address family (expected IPv6)"),
                    None => {
                        tracing::warn!("No IPv6 address was found for the default route interface");
                        None
                    }
                })
            })
            .transpose()
            .map_err(Error::LuidToIp)?
            .flatten();

        Ok(Self {
            tunnel_ipv4,
            tunnel_ipv6,
            internet_ipv4,
            internet_ipv6,
        })
    }
}

enum EventResult {
    /// Result containing the next event.
    Event(driver::EventId, driver::EventBody),
    /// Quit event was signaled.
    Quit,
}

enum Message {
    /// Set paths to exclude from the VPN tunnel
    SetExcludePaths {
        result_tx: oneshot::Sender<Result<(), Error>>,
        paths: HashSet<PathBuf>,
        hybrid_paths: HashSet<PathBuf>,
    },
    /// Update VPN tunnel interface
    SetTunnel {
        result_tx: oneshot::Sender<Result<(), Error>>,
        vpn_interface: VpnInterface,
    },
    /// Remove VPN tunnel interface. It is sufficient to call this when entering the disconnected
    /// state, to avoid pointless cleanup during reconnects.
    ResetTunnel {
        result_tx: oneshot::Sender<Result<(), Error>>,
    },
}

/// A process that is being excluded from the tunnel.
#[derive(Debug, Clone)]
pub struct ExcludedProcess {
    /// Process identifier.
    pub pid: u32,
    /// Path to the image that this process is an instance of.
    pub image: PathBuf,
    /// If true, then the process is split because its parent was split,
    /// not due to its path being in the config.
    pub inherited: bool,
}

/// Errors that may occur in [`SplitTunnel`].
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to install or start driver service
    #[error("Failed to start driver service")]
    ServiceError(#[source] service::Error),

    /// Failed to initialize the driver
    #[error("Failed to initialize driver")]
    InitializationError(#[source] driver::DeviceHandleError),

    /// Failed to reset the driver
    #[error("Failed to reset driver")]
    ResetError(#[source] io::Error),

    /// Failed to set paths to excluded applications
    #[error("Failed to set list of excluded applications")]
    SetConfiguration(#[source] io::Error),

    /// Failed to obtain the current driver state
    #[error("Failed to obtain the driver state")]
    GetState(#[source] io::Error),

    /// Failed to register interface IP addresses
    #[error("Failed to register IP addresses for exclusions")]
    RegisterIps(#[source] io::Error),

    /// Failed to clear interface IP addresses
    #[error("Failed to clear registered IP addresses")]
    ClearIps(#[source] io::Error),

    /// Failed to set up the driver event loop
    #[error("Failed to set up the driver event loop")]
    EventThreadError(#[source] io::Error),

    /// Failed to obtain default route
    #[error("Failed to obtain the default route")]
    ObtainDefaultRoute(#[source] nym_routing::Error),

    /// Failed to obtain an IP address given a network interface LUID
    #[error("Failed to obtain IP address for interface LUID")]
    LuidToIp(#[source] nym_windows::net::Error),

    /// Failed to set up callback for monitoring default route changes
    #[error("Failed to register default route change callback")]
    RegisterRouteChangeCallback,

    /// Unexpected IP parsing error
    #[error("Failed to parse IP address")]
    IpParseError,

    /// The request handling thread is stuck
    #[error("The ST request thread is stuck")]
    RequestThreadStuck,

    /// The request handling thread is down
    #[error("The split tunnel monitor is down")]
    SplitTunnelDown,

    /// Failed to start the NTFS reparse point monitor
    #[error("Failed to start path monitor")]
    StartPathMonitor(#[source] io::Error),

    /// A previous path update has not yet completed
    #[error("A previous update is not yet complete")]
    AlreadySettingPaths,

    /// Resetting in the engaged state risks leaking into the tunnel
    #[error("Failed to reset driver because it is engaged")]
    CannotResetEngaged,

    /// Split tunneling is unavailable
    #[error("Split tunneling is unavailable. Review logs for details.")]
    Unavailable,

    /// General error
    #[error("An error occurred: {0}.")]
    General(String),
}

impl From<&Error> for SplitTunnelErrorCause {
    fn from(_value: &Error) -> Self {
        Self::Other
    }
}
