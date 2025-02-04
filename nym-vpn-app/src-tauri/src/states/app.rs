use serde::{Deserialize, Serialize};
use tracing::{error, instrument};
use ts_rs::TS;

use crate::events::AppHandleEventEmitter;
use crate::grpc::tunnel::TunnelState;
use crate::{
    cli::Cli,
    db::{Db, Key},
    fs::config::AppConfig,
    grpc::client::{VpndInfo, VpndStatus},
};

#[derive(Default, Debug, Serialize, Deserialize, TS, Clone, PartialEq, Eq)]
#[ts(export)]
pub enum VpnMode {
    Mixnet,
    // ⚠ keep this default in sync with the one declared in
    // src/constants.ts
    #[default]
    TwoHop,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub vpnd_status: VpndStatus,
    pub vpnd_info: Option<VpndInfo>,
    pub tunnel: TunnelState,
    pub vpn_mode: VpnMode,
    pub dns_server: Option<String>,
    pub credentials_mode: bool,
}

impl AppState {
    pub fn new(db: &Db, config: &AppConfig, cli: &Cli) -> Self {
        let vpn_mode = db
            .get_typed::<VpnMode>(Key::VpnMode)
            .inspect_err(|e| error!("failed to retrieve vpn mode from db: {e}"))
            .ok()
            .flatten()
            .unwrap_or_default();
        let dns_server: Option<String> = cli.dns.clone().or(config.dns_server.clone());

        // restore any state from the saved app data (previous user session)
        AppState {
            vpn_mode,
            dns_server,
            credentials_mode: cli.dev_mode,
            ..Default::default()
        }
    }

    #[instrument(skip(self, app))]
    pub async fn update_tunnel(
        &mut self,
        app: &tauri::AppHandle,
        state: TunnelState,
    ) -> anyhow::Result<()> {
        self.tunnel = state;
        app.emit_tunnel_update(&self.tunnel);
        Ok(())
    }
}
