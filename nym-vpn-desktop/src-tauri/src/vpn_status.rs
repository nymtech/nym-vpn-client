use crate::events::{ConnectionEventPayload, EVENT_CONNECTION_STATE};
use crate::grpc::client::GrpcClient;
use crate::states::{app::ConnectionState, SharedAppState};
use anyhow::{anyhow, Result};
use nym_vpn_proto::nym_vpnd_client::NymVpndClient;
use nym_vpn_proto::StatusRequest;
use std::time::Duration;
use tauri::Manager;
use time::OffsetDateTime;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tonic::transport::Channel;
use tonic::Request;
use tracing::{debug, error, info, instrument, trace, warn};

const VPN_STATUS_POLL_INTERVAL: Duration = Duration::from_secs(1);

#[instrument(skip_all)]
async fn check_status_watchdog(app: &tauri::AppHandle) -> Result<bool> {
    let m_state = app
        .try_state::<SharedAppState>()
        .ok_or(anyhow!("no managed state"))?;

    let state = m_state.lock().await;
    if let Some(handle) = state.vpn_status_watchdog.as_ref() {
        return match handle.is_finished() {
            true => {
                info!("vpn status watchdog already running but finished");
                Ok(false)
            }
            false => {
                info!("vpn status watchdog already running");
                Ok(true)
            }
        };
    }

    info!("vpn status watchdog is not started yet");
    return Ok(false);
}

#[instrument(skip_all)]
fn vpn_status_watchdog_task(
    app: tauri::AppHandle,
    mut vpnd: NymVpndClient<Channel>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let state = app.state::<SharedAppState>();

        loop {
            let request = Request::new(StatusRequest {});
            let response = vpnd
                .vpn_status(request)
                .await
                .inspect_err(|e| {
                    error!("grpc vpn_status: {}", e);
                })
                .unwrap();

            let status = ConnectionState::from(response.get_ref().status());
            trace!("vpn status: {:?}", status);
            let error = response
                .get_ref()
                .error
                .clone()
                .map(|e| e.message)
                .inspect(|e| {
                    // TODO reduce logs spam as vpnd returns
                    // indefinitely the latest error
                    trace!("vpn status error: {e}");
                });

            let mut app_state = state.lock().await;
            let current_state = app_state.state.clone();
            app_state.state = status.clone();
            // release the lock asap
            drop(app_state);

            if current_state != status {
                error.as_ref().inspect(|e| warn!("vpn status error: {}", e));

                match status {
                    ConnectionState::Connected => {
                        info!("vpn status → [Connected]");
                        let now = OffsetDateTime::now_utc();
                        let mut app_state = state.lock().await;
                        app_state.state = status.clone();
                        app_state.connection_start_time = Some(now);
                        drop(app_state);
                        app.emit_all(
                            EVENT_CONNECTION_STATE,
                            ConnectionEventPayload::new(
                                ConnectionState::Connected,
                                error,
                                Some(now.unix_timestamp()),
                            ),
                        )
                        .unwrap();
                    }
                    ConnectionState::Disconnected => {
                        info!("vpn status → [Disconnected]");
                        let mut app_state = state.lock().await;
                        app_state.state = status.clone();
                        app_state.connection_start_time = None;
                        drop(app_state);
                        app.emit_all(
                            EVENT_CONNECTION_STATE,
                            ConnectionEventPayload::new(ConnectionState::Disconnected, error, None),
                        )
                        .unwrap();
                    }
                    ConnectionState::Connecting => {
                        info!("vpn status → [Connecting]");
                        app.emit_all(
                            EVENT_CONNECTION_STATE,
                            ConnectionEventPayload::new(ConnectionState::Connecting, error, None),
                        )
                        .unwrap();
                    }
                    ConnectionState::Disconnecting => {
                        info!("vpn status → [Disconnecting]");
                        app.emit_all(
                            EVENT_CONNECTION_STATE,
                            ConnectionEventPayload::new(
                                ConnectionState::Disconnecting,
                                error,
                                None,
                            ),
                        )
                        .unwrap();
                    }
                    ConnectionState::Unknown => {
                        warn!("vpn status → [Unknown]");
                        app.emit_all(
                            EVENT_CONNECTION_STATE,
                            ConnectionEventPayload::new(ConnectionState::Unknown, error, None),
                        )
                        .unwrap();
                    }
                }
            }
            sleep(VPN_STATUS_POLL_INTERVAL).await;
        }
    })
}

#[instrument(skip_all)]
pub async fn vpn_status_watchdog(app: &tauri::AppHandle, grpc_client: &GrpcClient) -> Result<()> {
    debug!("vpn_status_watchdog");

    if check_status_watchdog(app).await? {
        return Ok(());
    }

    let vpnd = grpc_client
        .vpnd()
        .inspect_err(|e| warn!("not connected to the daemon: {}", e))?;

    info!("starting vpn status watchdog");
    let handle = vpn_status_watchdog_task(app.clone(), vpnd);

    let m_state = app.state::<SharedAppState>();
    let mut app_state = m_state.lock().await;
    app_state.vpn_status_watchdog = Some(handle);
    Ok(())
}
