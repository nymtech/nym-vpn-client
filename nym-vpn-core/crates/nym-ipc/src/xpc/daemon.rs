// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::pin::Pin;

use objc2::{
    AnyThread, DefinedClass as _, define_class, msg_send, rc::Retained, runtime::ProtocolObject,
};
use objc2_foundation::{
    NSObject, NSObjectProtocol, NSString, NSXPCConnection, NSXPCInterface, NSXPCListener,
    NSXPCListenerDelegate,
};
use tokio::sync::mpsc;
use tokio_stream::{Stream, wrappers::UnboundedReceiverStream};
use tokio_util::sync::{CancellationToken, DropGuard};

use crate::{
    AuthenticationMaterial,
    authentication::{self, skip_authentication_checks},
    xpc::{
        common::{
            ConnectionInterfaceObj, DAEMON_BUNDLE_IDENTIFIER, NSConnectionInterface, XpcConnection,
            connection_interface,
        },
        local_spawner::LocalSpawner,
    },
};

#[derive(Clone)]
struct ListenerDelegateIvars {
    connection_interface: Retained<NSXPCInterface>,
    conn_sender: mpsc::UnboundedSender<XpcConnection>,
    signing_requirement: Option<String>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[ivars = ListenerDelegateIvars]
    struct ListenerDelegate;

    unsafe impl NSObjectProtocol for ListenerDelegate {}

    unsafe impl NSXPCListenerDelegate for ListenerDelegate {
        #[allow(non_snake_case)]
        #[unsafe(method(listener:shouldAcceptNewConnection:))]
        fn listener_shouldAcceptNewConnection(
            &self,
            _listener: &NSXPCListener,
            new_connection: &NSXPCConnection,
        ) -> bool {
            let (data_tx, data_rx) = mpsc::unbounded_channel();
            let exported_conn_int_obj = ConnectionInterfaceObj::new(data_tx);
            unsafe {
                new_connection.setExportedObject(Some(&exported_conn_int_obj));
            }

            new_connection.setExportedInterface(Some(&self.ivars().connection_interface));
            new_connection.setRemoteObjectInterface(Some(&self.ivars().connection_interface));

            let shutdown_token = CancellationToken::new();
            let shutdown_token_cloned = shutdown_token.clone();
            let invalidation_handler = block2::RcBlock::new(move || {
                shutdown_token_cloned.cancel();
            });
            new_connection.setInvalidationHandler(Some(&invalidation_handler));

            let proxy_obj = new_connection.remoteObjectProxy();
            let proxy = unsafe {
                Retained::cast_unchecked::<ProtocolObject<dyn NSConnectionInterface + Send + Sync>>(
                    proxy_obj,
                )
            };

            let xpc_conn = XpcConnection::new(proxy, data_rx.into(), shutdown_token);
            let forwarded = self.ivars().conn_sender.send(xpc_conn);

            if let Some(signing_requirement) = self.ivars().signing_requirement.as_ref() {
                new_connection.setCodeSigningRequirement(&NSString::from_str(signing_requirement));
            }

            new_connection.resume();

            if let Err(err) = forwarded {
                tracing::error!("Connection could not be forwarded: {err:?}");
                false
            } else {
                true
            }
        }
    }
);

impl ListenerDelegate {
    fn new(
        connection_interface: Retained<NSXPCInterface>,
        conn_sender: mpsc::UnboundedSender<XpcConnection>,
        signing_requirement: Option<String>,
    ) -> Retained<Self> {
        let this = Self::alloc().set_ivars(ListenerDelegateIvars {
            connection_interface,
            conn_sender,
            signing_requirement,
        });
        unsafe { msg_send![super(this), init] }
    }
}

enum Task {
    CreateListener {
        conn_sender: mpsc::UnboundedSender<XpcConnection>,
        signing_requirement: Option<String>,
    },
}

async fn create_listener(
    conn_sender: mpsc::UnboundedSender<XpcConnection>,
    signing_requirement: Option<String>,
    shutdown_token: CancellationToken,
) {
    let service_name = NSString::from_str(DAEMON_BUNDLE_IDENTIFIER);
    let listener = NSXPCListener::initWithMachServiceName(NSXPCListener::alloc(), &service_name);
    let interface = connection_interface();
    let listener_delegate = ListenerDelegate::new(interface, conn_sender, signing_requirement);
    let protocol_obj = ProtocolObject::from_retained(listener_delegate);
    listener.setDelegate(Some(&protocol_obj));
    listener.resume();
    tracing::info!("Started XPC listener");

    // Wait for shutdown, keeping the listener and protocol objects alive
    shutdown_token.cancelled().await;
    tracing::info!("Stopped XPC listener");
}

async fn run_task(task: Task, shutdown_token: CancellationToken) {
    match task {
        Task::CreateListener {
            conn_sender,
            signing_requirement,
        } => create_listener(conn_sender, signing_requirement, shutdown_token).await,
    }
}

pub(crate) struct XpcService {
    inner: UnboundedReceiverStream<XpcConnection>,
    // needed to keep the XPC listener object alive for the lifetime of this
    // service
    _drop_guard: DropGuard,
}

impl XpcService {
    pub(crate) fn spawn(auth_material: AuthenticationMaterial) -> std::io::Result<Self> {
        let local_spawner =
            LocalSpawner::new(run_task, auth_material.shutdown_token.child_token())?;
        let signing_requirement = if skip_authentication_checks(&auth_material) {
            tracing::debug!("Daemon will receive any XPC clients");
            None
        } else {
            tracing::debug!("Daemon will do code signing verification for clients");
            Some(auth_material.signing_requirements.client_req)
        };

        let (conn_sender, conn_receiver) = mpsc::unbounded_channel();
        local_spawner.spawn(Task::CreateListener {
            conn_sender,
            signing_requirement,
        });

        Ok(XpcService {
            inner: UnboundedReceiverStream::new(conn_receiver),
            _drop_guard: auth_material.shutdown_token.drop_guard(),
        })
    }
}

impl Stream for XpcService {
    type Item = std::io::Result<authentication::Transport>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner)
            .poll_next(cx)
            .map(|conn| conn.map(Ok))
    }
}

pub fn incoming(
    auth_material: AuthenticationMaterial,
) -> std::io::Result<impl Stream<Item = std::io::Result<authentication::Transport>>> {
    let xpc_service = XpcService::spawn(auth_material)?;
    Ok(authentication::incoming(xpc_service))
}
