use anyhow::Result;
use nym_vpn_lib_types::VpnServiceInfo;
use semver::{Version, VersionReq};
use serde::Serialize;
use tracing::error;
use ts_rs::TS;

#[derive(Serialize, Default, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct VpndInfo {
    pub version: String,
    pub network: String,
    pub git_commit: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub enum VpndStatus {
    /// Connected to the daemon
    Ok(Option<VpndInfo>),
    /// Connected to the daemon, but the version is not compatible with the client
    NonCompat {
        /// The current daemon info, including its version
        current: VpndInfo,
        /// The SemVer version requirement
        requirement: String,
    },
    /// The daemon is not serving or running
    Down,
    /// The daemon requires authentication that was denied or cancelled
    AuthDenied,
}

#[allow(clippy::derivable_impls)]
impl Default for VpndStatus {
    fn default() -> Self {
        #[cfg(target_os = "linux")]
        {
            VpndStatus::AuthDenied
        }

        #[cfg(any(target_os = "windows", target_os = "macos"))]
        {
            VpndStatus::Down
        }
    }
}

impl From<VpnServiceInfo> for VpndInfo {
    fn from(info: VpnServiceInfo) -> Self {
        VpndInfo {
            version: info.version.clone(),
            network: info.nym_network.network_name,
            git_commit: info.git_commit.clone(),
        }
    }
}

pub struct VersionCheck(VersionReq);

impl VersionCheck {
    pub fn new(req: &str) -> Result<Self> {
        let req = VersionReq::parse(req)
            .inspect_err(|e| error!("failed to parse version requirement [{req}]: {e}"))?;
        Ok(Self(req))
    }

    pub fn check(&self, version: &str) -> Result<bool> {
        let version = Version::parse(version)
            .inspect_err(|e| error!("failed to parse version [{version}]: {e}"))?;
        Ok(self.0.matches(&version))
    }
}
