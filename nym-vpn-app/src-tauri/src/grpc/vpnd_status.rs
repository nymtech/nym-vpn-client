use anyhow::Result;
use nym_vpn_proto::InfoResponse;
use semver::{Version, VersionReq};
use serde::Serialize;
use tracing::error;
use ts_rs::TS;

#[derive(Serialize, Default, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "DaemonInfo.ts")]
pub struct VpndInfo {
    pub version: String,
    pub network: String,
    pub git_commit: String,
}

#[derive(Serialize, Default, Clone, Debug, PartialEq, TS)]
#[ts(export)]
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
    #[default]
    Down,
}

impl From<&InfoResponse> for VpndInfo {
    fn from(info: &InfoResponse) -> Self {
        VpndInfo {
            version: info.version.clone(),
            network: info
                .nym_network
                .as_ref()
                .map(|network| network.network_name.to_owned())
                .unwrap_or_else(|| "unknown".to_string()),
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
