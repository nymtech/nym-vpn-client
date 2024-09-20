use std::fmt;

use talpid_core::firewall::{Firewall, FirewallArguments, InitialFirewallState};

pub use talpid_core::firewall::FirewallPolicy;

pub struct FirewallHandler {
    inner: Firewall,
}

impl FirewallHandler {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: Firewall::from_args(FirewallArguments {
                allow_lan: true,
                initial_state: InitialFirewallState::None,
                #[cfg(target_os = "linux")]
                fwmark: super::route_handler::TUNNEL_FWMARK,
            })?,
        })
    }

    // todo: hide firewall policy
    pub fn apply_policy(&mut self, policy: FirewallPolicy) -> Result<()> {
        Ok(self.inner.apply_policy(policy)?)
    }

    pub fn reset_policy(&mut self) -> Result<()> {
        Ok(self.inner.reset_policy()?)
    }
}

#[derive(Debug)]
pub struct Error {
    inner: talpid_core::firewall::Error,
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}

impl From<talpid_core::firewall::Error> for Error {
    fn from(value: talpid_core::firewall::Error) -> Self {
        Self { inner: value }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Firewall error")
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
