use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tracing::{debug, error, info, instrument, warn};
use ts_rs::TS;

use crate::env::{DEV_MODE, VPND_COMPAT_REQ};
use crate::events::AppHandleEventEmitter;
use crate::grpc::client::{NetworkCompatVersions, VersionCheck};
use crate::grpc::tunnel::TunnelState;
use crate::state::SharedAppState;
use crate::{
    cli::Cli,
    db::{Db, Key},
    fs::config::AppConfig,
    grpc::client::{VpndInfo, VpndStatus},
};

const NETWORK_VERSION_REQ_OP: &str = ">=";

#[derive(Default, Debug, Serialize, Deserialize, TS, Clone, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum VpnMode {
    Mixnet,
    // ⚠ keep this default in sync with the one declared in
    // src/constants.ts
    #[default]
    Wg,
}

#[derive(Default, Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct NetworkCompat {
    core: Option<bool>,
    tauri: Option<bool>,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub vpnd_status: VpndStatus,
    pub vpnd_info: Option<VpndInfo>,
    pub tunnel: TunnelState,
    pub vpn_mode: VpnMode,
    pub dns_server: Option<String>,
    pub credentials_mode: bool,
    pub network_compat: Option<NetworkCompat>,
}

impl AppState {
    pub fn new(db: &Db, config: &AppConfig, cli: &Cli) -> Self {
        let vpn_mode = db
            .get_typed::<VpnMode>(Key::VpnMode.as_ref())
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
    ) -> Result<()> {
        self.tunnel = state;
        app.emit_tunnel_update(&self.tunnel);
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn set_vpnd_status(&mut self, info: &VpndInfo) {
        let Some(ver_req) = VPND_COMPAT_REQ else {
            warn!("env variable `VPND_COMPAT_REQ` is not set, skipping vpnd version compatibility check");
            self.vpnd_status = VpndStatus::Ok(None);
            return;
        };
        let Ok(ver) = VersionCheck::new(ver_req) else {
            warn!("skipping vpnd version compatibility check");
            self.vpnd_status = VpndStatus::Ok(Some(info.to_owned()));
            return;
        };
        match ver.check(&info.version) {
            Ok(true) => {
                info!("daemon version compatibility check OK");
                self.vpnd_status = VpndStatus::Ok(Some(info.to_owned()));
            }
            Ok(false) => {
                warn!(
                    "daemon version is not compatible with the client, required [{}], version [{}]",
                    ver_req, info.version
                );
                self.vpnd_status = VpndStatus::NonCompat {
                    current: info.clone(),
                    requirement: ver_req.to_string(),
                };
            }
            Err(_) => {
                warn!("skipping vpnd version compatibility check");
                self.vpnd_status = VpndStatus::Ok(Some(info.to_owned()));
            }
        }
    }

    #[instrument(skip(self))]
    pub fn set_network_compat(
        &mut self,
        network_compat: Option<NetworkCompatVersions>,
        pkg_version: &semver::Version,
        info: &VpndInfo,
    ) {
        if *DEV_MODE {
            debug!("dev mode ON, skipping compatibility check");
            return;
        }

        let Some(compat) = network_compat else {
            warn!("no network compatibility data");
            return;
        };
        let core_compat = check_version(&compat.core, &info.version)
            .inspect_err(|e| warn!("failed to check core version: {e}"))
            .ok();
        log_compat(&info.version, &compat.core, core_compat, "core");

        let tauri_ver = pkg_version.to_string();
        let tauri_compat = check_version(&compat.tauri, &tauri_ver)
            .inspect_err(|e| warn!("failed to check tauri version: {e}"))
            .ok();

        log_compat(&tauri_ver, &compat.tauri, tauri_compat, "tauri");
        self.network_compat = Some(NetworkCompat::new(core_compat, tauri_compat));
    }

    #[instrument(skip_all)]
    pub async fn vpnd_down(app: &AppHandle) {
        let app_state = app.state::<SharedAppState>();
        let mut state = app_state.lock().await;
        if state.vpnd_status != VpndStatus::Down {
            warn!("vpnd DOWN");
            state.vpnd_status = VpndStatus::Down;
            app.emit_vpnd_status(state.vpnd_status.clone());
        }
    }
}

impl NetworkCompat {
    pub fn new(core: Option<bool>, tauri: Option<bool>) -> Self {
        NetworkCompat { core, tauri }
    }
}

#[instrument]
fn check_version(req: &str, version: &str) -> Result<bool> {
    let ver_check = VersionCheck::new(&format!("{NETWORK_VERSION_REQ_OP}{req}"))?;
    ver_check.check(version)
}

fn log_compat(local: &str, network: &str, is_compat: Option<bool>, comp_name: &str) {
    match is_compat {
        None => warn!("failed to check {comp_name} version compatibility, skipping"),
        Some(true) => info!("{comp_name} version is compatible with the network, local version: [{local}], network version: [{network}]"),
        Some(false) => warn!("{comp_name} version is not compatible with the network, local version: [{local}], network version: [{network}]")
    }
}
