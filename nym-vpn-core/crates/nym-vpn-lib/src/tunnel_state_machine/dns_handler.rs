// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_dns::{DnsMonitor, ResolvedDnsConfig};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

#[cfg(target_os = "linux")]
use super::route_handler::RouteHandler;

struct DnsHandler {
    inner: DnsMonitor,
}

impl DnsHandler {
    fn new(
        #[cfg(target_os = "linux")] route_handler: &RouteHandler,
    ) -> Result<Self, nym_dns::Error> {
        Ok(Self {
            inner: DnsMonitor::new(
                #[cfg(target_os = "linux")]
                route_handler.inner_handle(),
            )?,
        })
    }

    pub async fn set(
        &mut self,
        interface: &str,
        config: ResolvedDnsConfig,
    ) -> Result<(), nym_dns::Error> {
        self.inner.set(interface, config).await
    }

    pub async fn reset(&mut self) -> Result<(), nym_dns::Error> {
        self.inner.reset().await
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    pub async fn reset_before_interface_removal(&mut self) -> Result<(), nym_dns::Error> {
        self.inner.reset_before_interface_removal().await
    }
}

enum DnsHandlerCommand {
    Set {
        interface: String,
        config: ResolvedDnsConfig,
        reply_tx: oneshot::Sender<Result<(), nym_dns::Error>>,
    },
    #[allow(unused)]
    Reset {
        reply_tx: oneshot::Sender<Result<(), nym_dns::Error>>,
    },
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    ResetBeforeInterfaceRemoval {
        reply_tx: oneshot::Sender<Result<(), nym_dns::Error>>,
    },
}

#[derive(Debug, Clone)]
pub struct DnsHandlerHandle {
    tx: mpsc::UnboundedSender<DnsHandlerCommand>,
}

impl DnsHandlerHandle {
    pub fn spawn(
        #[cfg(target_os = "linux")] route_handler: &RouteHandler,
        shutdown_token: CancellationToken,
    ) -> Result<(Self, JoinHandle<()>)> {
        let mut dns_handler = DnsHandler::new(
            #[cfg(target_os = "linux")]
            route_handler,
        )?;

        let (tx, mut rx) = mpsc::unbounded_channel();
        let join_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(command) = rx.recv() => {
                        match command {
                            DnsHandlerCommand::Set {
                                interface,
                                config,
                                reply_tx,
                            } => {
                                _ = reply_tx.send(dns_handler.set(&interface, config).await);
                            }
                            DnsHandlerCommand::Reset { reply_tx } => {
                                _ = reply_tx.send(dns_handler.reset().await);
                            }
                            #[cfg(any(target_os = "linux", target_os = "windows"))]
                            DnsHandlerCommand::ResetBeforeInterfaceRemoval { reply_tx } => {
                                _ = reply_tx.send(dns_handler.reset_before_interface_removal().await);
                            }
                        }
                    }
                    _ = shutdown_token.cancelled() => break,
                    else => break
                }
            }
            tracing::debug!("Exiting dns handler loop");
        });

        Ok((Self { tx }, join_handle))
    }

    pub async fn set(&mut self, interface: String, config: ResolvedDnsConfig) -> Result<()> {
        let (reply_tx, reply_rx) = oneshot::channel();

        self.send_and_wait(
            DnsHandlerCommand::Set {
                interface,
                config,
                reply_tx,
            },
            reply_rx,
        )
        .await
    }

    pub async fn reset(&mut self) -> Result<()> {
        let (reply_tx, reply_rx) = oneshot::channel();

        self.send_and_wait(DnsHandlerCommand::Reset { reply_tx }, reply_rx)
            .await
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    pub async fn reset_before_interface_removal(&mut self) -> Result<()> {
        let (reply_tx, reply_rx) = oneshot::channel();

        self.send_and_wait(
            DnsHandlerCommand::ResetBeforeInterfaceRemoval { reply_tx },
            reply_rx,
        )
        .await
    }

    async fn send_and_wait<T>(
        &self,
        command: DnsHandlerCommand,
        reply_rx: oneshot::Receiver<Result<T, nym_dns::Error>>,
    ) -> Result<T> {
        self.tx.send(command).map_err(|_| Error::ChannelClosed)?;

        reply_rx
            .await
            .map_err(|_| Error::ChannelClosed)?
            .map_err(Error::DnsMonitor)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Dns monitor error: {_0}")]
    DnsMonitor(#[from] nym_dns::Error),

    #[error("Dns monitor is already down")]
    ChannelClosed,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
