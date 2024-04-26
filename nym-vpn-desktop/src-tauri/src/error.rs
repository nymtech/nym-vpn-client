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
    pub fn new(error: CmdErrorSource, message: &str) -> Self {
        Self {
            message: message.to_string(),
            source: error,
        }
    }
}

impl Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.source, self.message)
    }
}
