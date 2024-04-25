use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

#[derive(Error, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum CmdErrorSource {
    #[error("internal error")]
    InternalError,
    #[error("caller error")]
    CallerError,
    #[error("unknown error")]
    Unknown,
}

#[derive(Error, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CmdError {
    #[source]
    pub source: CmdErrorSource,
    pub message: String,
}

impl CmdError {
    pub fn new(error: CmdErrorSource, message: String) -> Self {
        Self {
            message,
            source: error,
        }
    }
}

impl From<nym_vpn_lib::error::Error> for CmdError {
    fn from(value: nym_vpn_lib::error::Error) -> Self {
        CmdError {
            source: CmdErrorSource::InternalError,
            message: value.to_string(),
        }
    }
}

impl From<nym_vpn_lib::gateway_directory::Error> for CmdError {
    fn from(value: nym_vpn_lib::gateway_directory::Error) -> Self {
        CmdError {
            source: CmdErrorSource::InternalError,
            message: value.to_string(),
        }
    }
}

impl Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.source, self.message)
    }
}
