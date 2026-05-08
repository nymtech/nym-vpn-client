use anyhow::Result;
use semver::Version;
use sentry::ClientInitGuard;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tracing::{debug, error, info, instrument, warn};
use ts_rs::TS;

use crate::env::{DEV_MODE, VPND_COMPAT_REQ};
use crate::events::AppHandleEventEmitter;
use crate::state::SharedAppState;
use crate::sys::OsInfo;
use crate::vpnd::account::AccountState;
use crate::vpnd::client::{NetworkCompatVersions, VersionCheck};
use crate::vpnd::tunnel::TunnelState;
use crate::{
    tray::TrayManager,
    vpnd::client::{VpndInfo, VpndStatus},
    vpnd::config::VpndConfig,
};

#[derive(Default, Debug, Serialize, Deserialize, TS, Clone, PartialEq, Eq, strum::Display)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "kebab-case")]
pub enum VpnMode {
    Mixnet,
    #[default]
    Wg,
}

#[derive(Default, Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct NetworkCompat {
    core: Option<bool>,
    tauri: Option<bool>,
}

// wrapper needed for Debug trait implem
#[derive(Default)]
pub struct SentryClient(pub Option<ClientInitGuard>);

#[derive(Debug, Default)]
pub struct AppState {
    pub os_info: OsInfo,
    pub vpnd_status: VpndStatus,
    pub vpnd_config: Option<VpndConfig>,
    pub vpnd_info: Option<VpndInfo>,
    pub tunnel: TunnelState,
    pub account_state: AccountState,
    pub network_compat: Option<NetworkCompat>,
    pub sentry_client: SentryClient,
}

impl AppState {
    pub fn new(os_info: OsInfo, sentry_guard: Option<ClientInitGuard>) -> Self {
        AppState {
            os_info,
            sentry_client: SentryClient(sentry_guard),
            ..Default::default()
        }
    }

    #[instrument(skip(self, app))]
    pub async fn update_vpnd_config(&mut self, app: &AppHandle, config: VpndConfig) -> Result<()> {
        self.vpnd_config = Some(config.clone());
        app.emit_vpnd_config(config);
        Ok(())
    }

    #[instrument(skip(self, app))]
    pub async fn update_tunnel(&mut self, app: &AppHandle, state: TunnelState) -> Result<()> {
        self.tunnel = state;

        let tray_manager = app.state::<TrayManager>();
        tray_manager.update_tray_icon(self.tunnel.clone()).await;
        app.emit_tunnel_update(&self.tunnel);
        Ok(())
    }

    #[instrument(skip(self, app))]
    pub async fn update_account_state(
        &mut self,
        app: &AppHandle,
        state: AccountState,
    ) -> Result<()> {
        self.account_state = state;
        app.emit_account_state_update(&self.account_state);
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn set_vpnd_status(&mut self, info: &VpndInfo) {
        let Some(ver_req) = VPND_COMPAT_REQ else {
            warn!(
                "env variable `VPND_COMPAT_REQ` is not set, skipping vpnd version compatibility check"
            );
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
        let core_compat = check_network_compat(&compat.core, &info.version)
            .inspect_err(|e| warn!("failed to check core version: {e}"))
            .ok();
        log_compat(&info.version, &compat.core, core_compat, "core");

        let tauri_ver = pkg_version.to_string();
        let tauri_compat = check_network_compat(&compat.tauri, &tauri_ver)
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
            info!("vpnd DOWN");
            state.vpnd_status = VpndStatus::Down;
            app.emit_vpnd_status(state.vpnd_status.clone());
        }
        let tray_manager = app.state::<TrayManager>();
        tray_manager
            .update_tray_icon(TunnelState::Offline { reconnect: false })
            .await;
    }

    #[instrument(skip_all)]
    pub async fn vpnd_auth_denied(app: &AppHandle) {
        let app_state = app.state::<SharedAppState>();
        let mut state = app_state.lock().await;
        if state.vpnd_status != VpndStatus::AuthDenied {
            info!("vpnd AUTH DENIED");
            state.vpnd_status = VpndStatus::AuthDenied;
            app.emit_vpnd_status(state.vpnd_status.clone());
        }
        let tray_manager = app.state::<TrayManager>();
        tray_manager
            .update_tray_icon(TunnelState::Offline { reconnect: false })
            .await;
    }
}

impl NetworkCompat {
    pub fn new(core: Option<bool>, tauri: Option<bool>) -> Self {
        NetworkCompat { core, tauri }
    }
}

#[instrument]
fn check_network_compat(network: &str, local: &str) -> Result<bool> {
    let network_ver = Version::parse(network).inspect_err(|e| {
        error!("failed to parse network version [{network}]: {e}");
    })?;
    let local_ver = Version::parse(local).inspect_err(|e| {
        error!("failed to parse local version [{local}]: {e}");
    })?;
    Ok(local_ver >= network_ver)
}

fn log_compat(local: &str, network: &str, is_compat: Option<bool>, comp_name: &str) {
    match is_compat {
        None => warn!("failed to check {comp_name} version compatibility, skipping"),
        Some(true) => info!(
            "{comp_name} version is compatible with the network, local version: [{local}], network version: [{network}]"
        ),
        Some(false) => warn!(
            "{comp_name} version is not compatible with the network, local version: [{local}], network version: [{network}]"
        ),
    }
}

impl std::fmt::Debug for SentryClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_some() {
            write!(f, "SentryClient(Some)")
        } else {
            write!(f, "SentryClient(None)")
        }
    }
}
