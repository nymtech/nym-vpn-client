use crate::events::{AppHandleEventEmitter, EVENT_CONNECTION_STATE};
use crate::states::{app::ConnectionState, SharedAppState};
use anyhow::Result;
use futures::channel::oneshot::Receiver as OneshotReceiver;
use futures::StreamExt;
use nym_vpn_lib::{NymVpnExitStatusMessage, StatusReceiver, TaskStatus};
use time::OffsetDateTime;
use tracing::{debug, error, info, instrument};

#[instrument(skip_all)]
pub async fn spawn_exit_listener(
    app: tauri::AppHandle,
    app_state: SharedAppState,
    exit_rx: OneshotReceiver<NymVpnExitStatusMessage>,
) -> Result<()> {
    tokio::spawn(async move {
        match exit_rx.await {
            Ok(res) => {
                debug!("received vpn exit message: {res:?}");
                match res {
                    NymVpnExitStatusMessage::Stopped => {
                        info!("vpn connection stopped");
                        debug!(
                            "vpn stopped, sending event [{}]: disconnected",
                            EVENT_CONNECTION_STATE
                        );
                        app.emit_disconnected(None);
                    }
                    NymVpnExitStatusMessage::Failed(e) => {
                        let error = e
                            .downcast::<nym_vpn_lib::error::Error>()
                            .unwrap_or(Box::new(nym_vpn_lib::error::Error::StopError));
                        debug!(
                            "vpn failed, sending event [{}]: disconnected",
                            EVENT_CONNECTION_STATE
                        );
                        app.emit_disconnected(Some(error.to_string()));
                    }
                }
            }
            Err(_) => {
                // This happens if there is a panic before the exit message is sent
                error!("vpn stopped before sending exit message (likely panic)");
                app.emit_disconnected(Some("vpn unexpected halt".to_string()));
            }
        }
        // update the connection state
        let mut state = app_state.lock().await;
        state.state = ConnectionState::Disconnected;
        state.connection_start_time = None;
        info!("vpn exit listener has exited");
    });
    Ok(())
}

#[instrument(skip_all)]
pub async fn spawn_status_listener(
    app: tauri::AppHandle,
    app_state: SharedAppState,
    mut status_rx: StatusReceiver,
) -> Result<()> {
    tokio::spawn(async move {
        while let Some(msg) = status_rx.next().await {
            debug!("received vpn status message: {msg:?}");
            if let Some(task_status) = msg.downcast_ref::<nym_vpn_lib::TaskStatus>() {
                let now = OffsetDateTime::now_utc();
                match task_status {
                    TaskStatus::Ready => {
                        info!("vpn connection has been established");
                        let gateway = "PLACEHOLDER".to_string();
                        app_state.lock().await.set_connected(now, gateway.clone());
                        app.emit_connected(now, gateway.clone());
                    }
                    TaskStatus::ReadyWithGateway(gateway) => {
                        info!("vpn connection has been established using entry gateway: {gateway}");
                        app_state.lock().await.set_connected(now, gateway.clone());
                        app.emit_connected(now, gateway.clone());
                    }
                }
            } else {
                info!("received unknown status message: {msg:?}");
            }
        }
        info!("vpn status listener has exited");
    });
    Ok(())
}
