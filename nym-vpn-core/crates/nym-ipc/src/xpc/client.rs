// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fmt::Debug,
    io::{Error, Result},
};

use objc2::{AnyThread, rc::Retained, runtime::ProtocolObject};
use objc2_foundation::{NSString, NSXPCConnection, NSXPCConnectionOptions};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use crate::xpc::{
    common::{
        ConnectionInterfaceObj, DAEMON_BUNDLE_IDENTIFIER, NSConnectionInterface, XpcConnection,
        connection_interface,
    },
    local_spawner::LocalSpawner,
};

enum Task {
    Connect(oneshot::Sender<XpcConnection>),
}

impl Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connect(_) => f.debug_tuple("Connect").finish(),
        }
    }
}

async fn xpc_connect(
    conn_sender: oneshot::Sender<XpcConnection>,
    shutdown_token: CancellationToken,
) {
    let service_name = NSString::from_str(DAEMON_BUNDLE_IDENTIFIER);
    let conn_obj = NSXPCConnection::initWithMachServiceName_options(
        NSXPCConnection::alloc(),
        &service_name,
        NSXPCConnectionOptions::Privileged,
    );

    let (data_tx, data_rx) = mpsc::unbounded_channel();
    let exported_conn_int_obj = ConnectionInterfaceObj::new(data_tx);
    unsafe {
        conn_obj.setExportedObject(Some(&exported_conn_int_obj));
    }

    let interface = connection_interface();
    conn_obj.setExportedInterface(Some(&interface));
    conn_obj.setRemoteObjectInterface(Some(&interface));

    let shutdown_token_cloned = shutdown_token.clone();
    let invalidation_handler = block2::RcBlock::new(move || {
        shutdown_token_cloned.cancel();
    });
    conn_obj.setInvalidationHandler(Some(&invalidation_handler));

    let proxy_obj = conn_obj.remoteObjectProxy();
    let proxy = unsafe {
        Retained::cast_unchecked::<ProtocolObject<dyn NSConnectionInterface + Send + Sync>>(
            proxy_obj,
        )
    };

    conn_obj.resume();

    let xpc_conn = XpcConnection::new(proxy, data_rx.into(), shutdown_token.clone());
    // The receiver is waiting in XpcClient::connect
    conn_sender.send(xpc_conn).ok();

    // Wait for shutdown, keeping the connection object alive
    shutdown_token.cancelled().await;
}

async fn run_task(task: Task, shutdown_token: CancellationToken) {
    match task {
        Task::Connect(conn_sender) => xpc_connect(conn_sender, shutdown_token).await,
    }
}

pub async fn connect() -> Result<XpcConnection> {
    let shutdown_token = CancellationToken::new();
    let (conn_sender, conn_receiver) = oneshot::channel();

    let local_spawner = LocalSpawner::new(run_task, shutdown_token.child_token())?;
    local_spawner.spawn(Task::Connect(conn_sender));
    conn_receiver
        .await
        .map(|c| c.with_drop_guard(shutdown_token.drop_guard()))
        .or(Err(Error::other("XPC connection setup failed")))
}
