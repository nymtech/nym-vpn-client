use crate::events::{ConnectionEventPayload, EVENT_CONNECTION_STATE};
use crate::states::{app::ConnectionState, SharedAppState};
use anyhow::Result;
use nym_vpn_proto::ConnectionStateChange;
use tauri::Manager;
use time::OffsetDateTime;
use tracing::{info, instrument, trace, warn};

#[instrument(skip_all)]
pub async fn vpn_status_update(
    app: &tauri::AppHandle,
    update: ConnectionStateChange,
) -> Result<()> {
    let state = app.state::<SharedAppState>();
    let status = ConnectionState::from(update.status());
    trace!("vpn status: {:?}", status);
    let error = update.error.clone().map(|e| e.message);

    let mut app_state = state.lock().await;
    let current_state = app_state.state.clone();
    app_state.state = status.clone();
    // release the lock asap
    drop(app_state);

    if current_state == status {
        return Ok(());
    }
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
            .ok();
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
            .ok();
        }
        ConnectionState::Connecting => {
            info!("vpn status → [Connecting]");
            app.emit_all(
                EVENT_CONNECTION_STATE,
                ConnectionEventPayload::new(ConnectionState::Connecting, error, None),
            )
            .ok();
        }
        ConnectionState::Disconnecting => {
            info!("vpn status → [Disconnecting]");
            app.emit_all(
                EVENT_CONNECTION_STATE,
                ConnectionEventPayload::new(ConnectionState::Disconnecting, error, None),
            )
            .ok();
        }
        ConnectionState::Unknown => {
            warn!("vpn status → [Unknown]");
            app.emit_all(
                EVENT_CONNECTION_STATE,
                ConnectionEventPayload::new(ConnectionState::Unknown, error, None),
            )
            .ok();
        }
    }
    Ok(())
}
