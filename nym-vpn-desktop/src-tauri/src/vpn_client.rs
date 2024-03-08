use crate::events::{AppHandleEventEmitter, EVENT_CONNECTION_STATE};
use crate::states::{app::ConnectionState, SharedAppState};
use anyhow::Result;
use futures::channel::oneshot::Receiver as OneshotReceiver;
use futures::StreamExt;
use nym_vpn_lib::gateway_client::{Config as GatewayClientConfig, EntryPoint, ExitPoint};
use nym_vpn_lib::nym_config::defaults::var_names::{EXPLORER_API, NYM_API};
use nym_vpn_lib::nym_config::OptionalSet;
use nym_vpn_lib::{NymVpn, NymVpnExitError, NymVpnExitStatusMessage, StatusReceiver, TaskStatus};
use time::OffsetDateTime;
use tracing::{debug, error, info, instrument};

fn handle_vpn_exit_error(e: Box<dyn std::error::Error + Send + Sync>) -> String {
    match e.downcast::<Box<NymVpnExitError>>() {
        Ok(e) => {
            // TODO The double boxing here is unexpected, we should look into that
            match **e {
                NymVpnExitError::Generic { reason } => reason.to_string(),
                NymVpnExitError::FailedToResetFirewallPolicy { reason } => reason.to_string(),
                NymVpnExitError::FailedToResetDnsMonitor { reason } => reason.to_string(),
            }
        }
        Err(e) => format!("unknown error: {e}"),
    }
}

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
                        let error = handle_vpn_exit_error(e);
                        debug!(
                            "vpn failed, sending event [{}]: disconnected",
                            EVENT_CONNECTION_STATE
                        );
                        app.emit_disconnected(Some(error));
                    }
                }
            }
            Err(e) => {
                error!("vpn_exit_rx failed to receive exit message: {}", e);
                app.emit_disconnected(Some("failed to receive exit message".to_string()));
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
                        panic!("received unexpected Ready status message");
                    }
                    TaskStatus::ReadyWithGateway(gateway) => {
                        info!("vpn connection has been established to gatway: {gateway}");
                        app_state.lock().await.set_connected(now, gateway.clone());
                        app.emit_connected(now, gateway.clone());
                    }
                }
            } else {
                error!("received unknown status message: {msg:?}");
            }
        }
        info!("vpn status listener has exited");
    });
    Ok(())
}

fn setup_gateway_client_config(private_key: Option<&str>) -> GatewayClientConfig {
    let mut config = GatewayClientConfig::default()
        // Read in the environment variable NYM_API if it exists
        .with_optional_env(GatewayClientConfig::with_custom_api_url, None, NYM_API)
        .with_optional_env(
            GatewayClientConfig::with_custom_explorer_url,
            None,
            EXPLORER_API,
        );
    info!("Using nym-api: {}", config.api_url());

    if let Some(key) = private_key {
        config = config.with_local_private_key(key.into());
    }
    config
}

#[instrument(skip_all)]
pub fn create_vpn_config(entry_point: EntryPoint, exit_point: ExitPoint) -> NymVpn {
    let mut nym_vpn = NymVpn::new(entry_point, exit_point);
    nym_vpn.gateway_config = setup_gateway_client_config(None);
    nym_vpn
}
