use anyhow::Result;
use semver::{Version, VersionReq};
use tracing::error;

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
