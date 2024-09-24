use std::{fmt, net::IpAddr, sync::Arc};

use futures::{channel::mpsc::UnboundedSender, StreamExt};
use talpid_core::dns::DnsMonitor;

#[cfg(target_os = "linux")]
use super::route_handler::RouteHandler;

#[cfg(target_os = "macos")]
type MullvadTunnelCommand = talpid_core::tunnel_state_machine::TunnelCommand;

pub struct DnsHandler {
    inner: DnsMonitor,

    /// Internal sender a weak reference to which is passed into `talpid_core::dns::DnsMonitor`.
    /// It must be retained throughout the lifetime of `DnsHandler`.
    #[cfg(target_os = "macos")]
    _tx: Arc<UnboundedSender<MullvadTunnelCommand>>,
}

impl DnsHandler {
    pub async fn new(#[cfg(target_os = "linux")] route_handler: &RouteHandler) -> Result<Self> {
        #[cfg(target_os = "macos")]
        let tx = {
            let (tx, mut rx) = futures::channel::mpsc::unbounded();

            tokio::spawn(async move {
                while let Some(cmd) = rx.next().await {
                    if let MullvadTunnelCommand::Block(_) = cmd {
                        tracing::debug!(
                            "Failed to set dns at runtime caused by a burst of changes to dns"
                        );
                        // todo: bubble error to consumer
                    }
                }
            });

            Arc::new(tx)
        };

        Ok(Self {
            inner: DnsMonitor::new(
                #[cfg(target_os = "linux")]
                tokio::runtime::Handle::current(),
                #[cfg(target_os = "linux")]
                route_handler.inner_handle()?,
                #[cfg(target_os = "macos")]
                Arc::downgrade(&tx),
            )?,
            #[cfg(target_os = "macos")]
            _tx: tx,
        })
    }

    pub fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        Ok(self.inner.set(interface, servers)?)
    }

    pub fn reset(&mut self) -> Result<()> {
        Ok(self.inner.reset()?)
    }

    pub fn reset_before_interface_removal(&mut self) -> Result<()> {
        Ok(self.inner.reset_before_interface_removal()?)
    }
}

#[derive(Debug)]
pub struct Error {
    inner: talpid_core::dns::Error,
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}

impl From<talpid_core::dns::Error> for Error {
    fn from(value: talpid_core::dns::Error) -> Self {
        Self { inner: value }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DNS error")
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
